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
        let target = record.target();
        if record.level() <= log::Level::Info
            || msg.contains("COVER")
            || msg.contains("Modal")
            || target.contains("day")
            || target.contains("eschool")
        {
            nslog::nslog(&format!("{:<5} {}: {}", record.level(), target, msg));
        }
    }

    fn flush(&self) {}
}

static BRIDGE: Bridge = Bridge;

pub fn install() {
    let _ = log::set_logger(&BRIDGE);
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);
}
