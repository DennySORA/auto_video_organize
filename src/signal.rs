use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[must_use]
pub fn setup_shutdown_signal() -> Arc<AtomicBool> {
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let signal_clone = Arc::clone(&shutdown_signal);

    ctrlc::set_handler(move || {
        signal_clone.store(true, Ordering::SeqCst);
        eprintln!("\n收到中斷信號，正在安全關閉...");
    })
    .expect("無法設定 Ctrl-C 處理器");

    shutdown_signal
}
