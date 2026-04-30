use std::path::Path;

fn load_dotenv() {
    // Try .env file first (local development)
    for path in &["../../.env", "../.env", ".env"] {
        if Path::new(path).exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();
                        println!("cargo:rustc-env={}={}", key, value);
                    }
                }
            }
            return;
        }
    }

    // Fallback: read from process environment (CI builds)
    let keys = [
        "OAUTH_CLIENT_ID",
        "OAUTH_CLIENT_SECRET",
        "GEMINI_OAUTH_CLIENT_ID",
        "GEMINI_OAUTH_CLIENT_SECRET",
    ];
    for key in &keys {
        if let Ok(value) = std::env::var(key) {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../../.env");
    println!("cargo:rerun-if-changed=build.rs");
    load_dotenv();
}
