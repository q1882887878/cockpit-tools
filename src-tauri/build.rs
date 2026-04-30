#[cfg(target_os = "macos")]
use swift_rs::SwiftLinker;

#[cfg(target_os = "macos")]
fn link_macos_swift_runtime_rpaths() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}

fn load_dotenv() {
    // Try .env file first (local development)
    for path in &["../.env", ".env"] {
        if std::path::Path::new(path).exists() {
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
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../.env");
    load_dotenv();

    #[cfg(target_os = "macos")]
    {
        SwiftLinker::new("12.0")
            .with_package("MacosNativeMenuSwift", "native/macos-native-menu")
            .link();
        link_macos_swift_runtime_rpaths();
    }

    tauri_build::build()
}
