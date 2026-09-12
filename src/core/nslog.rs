//! Device logging — writes to ~/Documents/eschool.log
//! Read via: ssh root@<iphone> cat /var/mobile/Containers/Data/Application/<UUID>/Documents/eschool.log

pub fn nslog(msg: &str) {
    let tagged = format!("ESCHOOL: {msg}\n");

    // Try HOME/Documents first (standard iOS sandbox)
    if let Ok(home) = std::env::var("HOME") {
        let path = format!("{home}/Documents/eschool.log");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            use std::io::Write;
            let _ = f.write_all(tagged.as_bytes());
            return;
        }
    }

    // Fallback: /var/tmp
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/tmp/eschool.log")
    {
        use std::io::Write;
        let _ = f.write_all(tagged.as_bytes());
    }
}
