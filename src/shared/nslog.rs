//! Device logging via NSLog bridge → idevicesyslog.
//! Also buffers logs for in-app dev viewer.

use std::sync::Mutex;

static LOG_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
const MAX_LOGS: usize = 200;

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn eschool_nslog(msg: *const i8);
}

#[cfg(target_os = "ios")]
pub fn nslog(msg: &str) {
    let tagged = format!("ESCHOOL: {msg}\0");
    push_log(msg);
    unsafe {
        eschool_nslog(tagged.as_ptr() as *const i8);
    }
}

#[cfg(not(target_os = "ios"))]
pub fn nslog(msg: &str) {
    eprintln!("ESCHOOL: {msg}");
    push_log(msg);
}

fn push_log(msg: &str) {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() % 10000)
            .unwrap_or(0);
        buf.push(format!("[{ts}] {msg}"));
        if buf.len() > MAX_LOGS {
            let excess = buf.len() - MAX_LOGS;
            buf.drain(0..excess);
        }
    }
}

pub fn get_logs() -> String {
    LOG_BUFFER.lock()
        .map(|buf| buf.join("\n"))
        .unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.clear();
    }
}
