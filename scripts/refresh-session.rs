use reqwest::header::{HeaderMap, HeaderValue};

const OAUTH: &str = "https://oauth.rios.unibel.by";
const TOKEN_FILE: &str = concat!(env!("HOME"), "/.eschool-tokens.json");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let refresh = if args.get(1).map(|s| s.as_str()) == Some("--") {
        std::fs::read_to_string(TOKEN_FILE).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("refresh_token").and_then(|r| r.as_str()).map(String::from))
    } else {
        args.get(1).cloned()
    }.expect("usage: refresh-tool <refresh_token>");

    // Try with X-Forwarded-For to mimic client IP
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert("X-Forwarded-For", HeaderValue::from_static("93.125.72.100"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)"));

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Method 1: standard form
    eprintln!("Method 1: POST /connect/token (form)...");
    let resp = client.post(format!("{}/connect/token", OAUTH))
        .form(&[
            ("client_id", "oauth_diary_echools"),
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
        ])
        .send();
    match resp {
        Ok(r) => {
            eprintln!("Status: {}", r.status());
            if r.status().is_success() {
                let body: serde_json::Value = r.json().unwrap();
                println!("{}", serde_json::to_string_pretty(&body).unwrap());
            } else {
                eprintln!("Body: {}", r.text().unwrap_or_default());
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    // Method 2: JSON body
    eprintln!("\nMethod 2: POST /connect/token (json)...");
    let resp2 = client.post(format!("{}/connect/token", OAUTH))
        .json(&serde_json::json!({
            "client_id": "oauth_diary_echools",
            "grant_type": "refresh_token",
            "refresh_token": refresh,
        }))
        .send();
    match resp2 {
        Ok(r) => {
            eprintln!("Status: {}", r.status());
            if r.status().is_success() {
                let body: serde_json::Value = r.json().unwrap();
                println!("{}", serde_json::to_string_pretty(&body).unwrap());
            } else {
                eprintln!("Body: {}", r.text().unwrap_or_default());
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
