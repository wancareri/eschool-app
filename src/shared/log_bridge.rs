use crate::shared::nslog;

struct Bridge;

impl log::Log for Bridge {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let msg = record.args().to_string();
        if record.level() <= log::Level::Warn || msg.contains("COVERDBG") {
            nslog::nslog(&format!("{:<5} {}: {}", record.level(), record.target(), msg));
        }
    }

    fn flush(&self) {}
}

static BRIDGE: Bridge = Bridge;

pub fn install() {
    let _ = log::set_logger(&BRIDGE);
    log::set_max_level(log::LevelFilter::Debug);
}
