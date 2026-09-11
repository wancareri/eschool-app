//! Minimal device logging — writes to /tmp/eschool.log and stderr.
//! No FFI, no framework linking, just plain Rust std.

pub fn nslog(msg: &str) {
    let tagged = format!("ESCHOOL: {msg}");
    // stderr (may show in some tools)
    eprintln!("{tagged}");
    // File (readable via `idevicesyslog` grep or iTunes file sharing)
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/eschool.log")
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{}", tagged)
        });
}
