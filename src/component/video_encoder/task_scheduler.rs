use super::cpu_monitor::CpuMonitor;
use super::ffmpeg_command::FfmpegCommand;
use crate::config::PostEncodeAction;
use crate::tools::{VideoFileInfo, ensure_directory_exists};
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fs, thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct EncodingTask {
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub status: TaskStatus,
    pub error_message: Option<String>,
}

impl EncodingTask {
    #[must_use]
    pub fn new(video_info: &VideoFileInfo) -> Self {
        let ffmpeg_cmd = FfmpegCommand::new(&video_info.path);
        Self {
            source_path: video_info.path.clone(),
            destination_path: ffmpeg_cmd.destination_path().to_path_buf(),
            status: TaskStatus::Pending,
            error_message: None,
        }
    }
}

struct RunningProcess {
    child: Child,
    task_index: usize,
    destination_path: PathBuf,
}

pub struct TaskScheduler {
    tasks: Vec<EncodingTask>,
    running_processes: HashMap<u32, RunningProcess>,
    cpu_monitor: CpuMonitor,
    shutdown_signal: Arc<AtomicBool>,
    fail_directory: PathBuf,
    finish_directory: PathBuf,
    post_encode_action: PostEncodeAction,
}

impl TaskScheduler {
    pub fn new(
        video_files: Vec<VideoFileInfo>,
        base_directory: &Path,
        shutdown_signal: Arc<AtomicBool>,
        post_encode_action: PostEncodeAction,
    ) -> Result<Self> {
        let fail_directory = base_directory.join("fail");
        let finish_directory = base_directory.join("finish");
        ensure_directory_exists(&fail_directory)?;

        // 只有在需要時才建立 finish 目錄
        if post_encode_action != PostEncodeAction::None {
            ensure_directory_exists(&finish_directory)?;
        }

        let tasks = video_files.iter().map(EncodingTask::new).collect();

        Ok(Self {
            tasks,
            running_processes: HashMap::new(),
            cpu_monitor: CpuMonitor::default(),
            shutdown_signal,
            fail_directory,
            finish_directory,
            post_encode_action,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("開始編碼任務，共 {} 個檔案", self.tasks.len());

        while !self.is_all_completed() {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                self.handle_shutdown()?;
                return Ok(());
            }

            self.check_completed_processes()?;
            self.spawn_new_tasks_if_possible()?;
            self.print_status();

            thread::sleep(Duration::from_secs(1));
        }

        info!("所有編碼任務已完成");
        Ok(())
    }

    fn is_all_completed(&self) -> bool {
        self.tasks
            .iter()
            .all(|t| matches!(t.status, TaskStatus::Completed | TaskStatus::Failed))
            && self.running_processes.is_empty()
    }

    fn spawn_new_tasks_if_possible(&mut self) -> Result<()> {
        while self.cpu_monitor.can_spawn_new_task() {
            if let Some(task_index) = self.find_next_pending_task() {
                self.spawn_task(task_index)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn find_next_pending_task(&self) -> Option<usize> {
        self.tasks
            .iter()
            .position(|t| t.status == TaskStatus::Pending)
    }

    fn spawn_task(&mut self, task_index: usize) -> Result<()> {
        let task = &mut self.tasks[task_index];
        let ffmpeg_cmd = FfmpegCommand::new(&task.source_path);

        let mut command = ffmpeg_cmd.build_command();
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        match command.spawn() {
            Ok(child) => {
                let pid = child.id();
                task.status = TaskStatus::Running;

                info!(
                    "啟動編碼任務 [{}]: {} -> {}",
                    pid,
                    task.source_path.display(),
                    task.destination_path.display()
                );

                self.running_processes.insert(
                    pid,
                    RunningProcess {
                        child,
                        task_index,
                        destination_path: task.destination_path.clone(),
                    },
                );
            }
            Err(e) => {
                task.status = TaskStatus::Failed;
                task.error_message = Some(format!("無法啟動 ffmpeg: {e}"));
                error!("無法啟動編碼任務: {e}");
            }
        }

        Ok(())
    }

    fn check_completed_processes(&mut self) -> Result<()> {
        let mut completed_pids = Vec::new();

        for (pid, process) in &mut self.running_processes {
            match process.child.try_wait() {
                Ok(Some(status)) => {
                    completed_pids.push((*pid, status.success()));
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("無法檢查程序狀態 [{pid}]: {e}");
                    completed_pids.push((*pid, false));
                }
            }
        }

        for (pid, exit_success) in completed_pids {
            if let Some(mut process) = self.running_processes.remove(&pid) {
                let task = &mut self.tasks[process.task_index];

                // 檢查輸出檔案是否存在且有效（大於 1KB）
                let output_valid = task.destination_path.exists()
                    && fs::metadata(&task.destination_path)
                        .map(|m| m.len() > 1024)
                        .unwrap_or(false);

                if exit_success {
                    task.status = TaskStatus::Completed;
                    info!("編碼完成 [{}]: {}", pid, task.destination_path.display());

                    if let Err(e) = self.handle_post_encode_action(process.task_index) {
                        warn!("轉檔後處理失敗: {}", e);
                    }
                } else if output_valid {
                    // FFmpeg 退出碼非零但輸出檔案有效，視為成功（來源檔可能有損壞的 frame）
                    task.status = TaskStatus::Completed;
                    warn!(
                        "編碼完成但有警告 [{}]: {} (來源檔案可能有損壞的 frame)",
                        pid,
                        task.destination_path.display()
                    );

                    if let Err(e) = self.handle_post_encode_action(process.task_index) {
                        warn!("轉檔後處理失敗: {}", e);
                    }
                } else {
                    let stderr = process.child.stderr.take();
                    let error_msg = stderr
                        .map(|s| {
                            BufReader::new(s)
                                .lines()
                                .map_while(Result::ok)
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_else(|| "未知錯誤".to_string());

                    task.status = TaskStatus::Failed;
                    task.error_message = Some(error_msg.clone());
                    error!("編碼失敗 [{pid}]: {error_msg}");

                    self.handle_failed_task(process.task_index)?;
                }
            }
        }

        Ok(())
    }

    fn handle_failed_task(&self, task_index: usize) -> Result<()> {
        let task = &self.tasks[task_index];

        if task.destination_path.exists() {
            fs::remove_file(&task.destination_path).with_context(|| {
                format!(
                    "無法刪除失敗的輸出檔案: {}",
                    task.destination_path.display()
                )
            })?;
            info!("已刪除失敗的輸出檔案: {}", task.destination_path.display());
        }

        let file_name = task
            .source_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("無法取得檔案名稱"))?;
        let fail_path = self.fail_directory.join(file_name);

        fs::rename(&task.source_path, &fail_path).with_context(|| {
            format!(
                "無法移動失敗檔案到 fail 資料夾: {} -> {}",
                task.source_path.display(),
                fail_path.display()
            )
        })?;

        info!(
            "已移動失敗的原始檔案到 fail 資料夾: {}",
            fail_path.display()
        );

        Ok(())
    }

    /// 處理轉檔成功後的動作
    fn handle_post_encode_action(&self, task_index: usize) -> Result<()> {
        let task = &self.tasks[task_index];

        match self.post_encode_action {
            PostEncodeAction::None => {
                // 不做任何動作
                Ok(())
            }
            PostEncodeAction::MoveOldToFinish => {
                // 移動舊影片（原始檔案）到 finish 資料夾
                let file_name = task
                    .source_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("無法取得檔案名稱"))?;
                let finish_path = self.finish_directory.join(file_name);

                fs::rename(&task.source_path, &finish_path).with_context(|| {
                    format!(
                        "無法移動原始檔案到 finish 資料夾: {} -> {}",
                        task.source_path.display(),
                        finish_path.display()
                    )
                })?;

                info!("已移動原始檔案到 finish 資料夾: {}", finish_path.display());
                Ok(())
            }
            PostEncodeAction::MoveNewToFinish => {
                // 移動新影片（轉檔後檔案）到 finish 資料夾
                let file_name = task
                    .destination_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("無法取得檔案名稱"))?;
                let finish_path = self.finish_directory.join(file_name);

                fs::rename(&task.destination_path, &finish_path).with_context(|| {
                    format!(
                        "無法移動轉檔檔案到 finish 資料夾: {} -> {}",
                        task.destination_path.display(),
                        finish_path.display()
                    )
                })?;

                info!("已移動轉檔檔案到 finish 資料夾: {}", finish_path.display());
                Ok(())
            }
        }
    }

    fn handle_shutdown(&mut self) -> Result<()> {
        warn!("收到中斷信號，正在停止所有任務...");

        for (pid, mut process) in self.running_processes.drain() {
            warn!("終止程序 [{pid}]");
            let _ = process.child.kill();
            let _ = process.child.wait();

            if process.destination_path.exists() {
                if let Err(e) = fs::remove_file(&process.destination_path) {
                    error!(
                        "無法刪除中斷的輸出檔案 {}: {}",
                        process.destination_path.display(),
                        e
                    );
                } else {
                    info!(
                        "已刪除中斷的輸出檔案: {}",
                        process.destination_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    fn print_status(&self) {
        let pending = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();
        let running = self.running_processes.len();
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();

        println!(
            "\r\x1b[K[狀態] 等待: {} | 執行中: {} | 完成: {} | 失敗: {} | CPU: {:.1}%",
            pending,
            running,
            completed,
            failed,
            self.cpu_monitor.system.global_cpu_usage()
        );
    }

    #[must_use]
    pub fn tasks(&self) -> &[EncodingTask] {
        &self.tasks
    }
}
