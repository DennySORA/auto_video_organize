use std::thread;
use std::time::Duration;
use sysinfo::System;

pub struct CpuMonitor {
    pub system: System,
    usage_threshold: f32,
}

impl CpuMonitor {
    #[must_use]
    pub fn new(usage_threshold: f32) -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_all();
        thread::sleep(Duration::from_millis(200));
        system.refresh_cpu_all();
        Self {
            system,
            usage_threshold,
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_cpu_all();
    }

    pub fn current_usage(&mut self) -> f32 {
        self.refresh();
        self.system.global_cpu_usage()
    }

    pub fn can_spawn_new_task(&mut self) -> bool {
        self.current_usage() < self.usage_threshold
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new(95.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_monitor_creation() {
        let monitor = CpuMonitor::new(80.0);
        assert_eq!(monitor.usage_threshold, 80.0);
    }
}
