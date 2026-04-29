use std::path::Path;

fn load_dotenv() {
    // Look for .env in workspace root (../../ from crate dir)
    let candidates = ["../../.env", "../.env", ".env"];
    for path in &candidates {
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
            break;
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../../.env");
    load_dotenv();
}
