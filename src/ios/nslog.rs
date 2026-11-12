//! Device logging via NSLog bridge → idevicesyslog.
//! Also buffers logs for in-app dev viewer.

use std::sync::Mutex;

static LOG_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
const MAX_LOGS: usize = 1500;

#[cfg(feature = "uikit")]
unsafe extern "C" {
    fn eschool_nslog(msg: *const i8);
}

#[cfg(feature = "uikit")]
pub fn nslog(msg: &str) {
    let tagged = format!("ESCHOOL: {msg}\0");
    push_log(msg);
    unsafe {
        eschool_nslog(tagged.as_ptr() as *const i8);
    }
}

#[cfg(not(feature = "uikit"))]
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
    LOG_BUFFER
        .lock()
        .map(|buf| buf.join("\n"))
        .unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.clear();
    }
}

/// Export only the dedicated trait diagnostics, excluding general app/API logs.
pub fn autofill_diagnostics() -> String {
    filter_autofill_diagnostics(&get_logs())
}

fn filter_autofill_diagnostics(logs: &str) -> String {
    logs.lines()
        .filter(|line| {
            line.split_once("] ")
                .is_some_and(|(_, message)| message.starts_with("[AutoFill] "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod diagnostic_tests {
    use super::filter_autofill_diagnostics;

    #[test]
    fn export_excludes_general_logs_and_embedded_autofill_tags() {
        let logs = "[1] [Auth] token=private\n[2] [AutoFill] role=Username language=ru\n[3] [Auth] response contains [AutoFill] password=private\n[4] [AutoFill] role=Password language=en\n";
        assert_eq!(
            filter_autofill_diagnostics(logs),
            "[2] [AutoFill] role=Username language=ru\n[4] [AutoFill] role=Password language=en"
        );
        assert_eq!(filter_autofill_diagnostics("[5] [API] private"), "");
    }
}
