/// 许可证验证模块
///
/// 验证流程：
/// 1. 启动时检查本地是否已保存有效许可证
/// 2. 未激活则通知前端显示激活窗口
/// 3. 用户输入授权码后，后端校验并保存
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Emitter;

use crate::get_app_handle;
use super::logger;

/// 授权码盐值（硬编码，编译后不可见）
const LICENSE_SEED: &str = "ctla-v4x9-2026pqz";

/// 许可证数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub machine_id: String,
    pub license_code: String,
    pub activated_at: i64,
}

/// 全局激活状态
static LICENSE_ACTIVATED: OnceLock<bool> = OnceLock::new();

/// 获取许可证存储路径
fn license_path() -> Option<PathBuf> {
    let dir = crate::modules::config::get_shared_dir();
    Some(dir.join(".license.dat"))
}

/// 获取机器唯一标识
fn get_machine_id() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    #[cfg(target_os = "windows")]
    let extra = {
        let vol = std::process::Command::new("wmic")
            .args(["csproduct", "get", "uuid"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        format!("{}-{}", hostname, vol.trim())
    };

    #[cfg(not(target_os = "windows"))]
    let extra = hostname.clone();

    let hash = Sha256::digest(extra.as_bytes());
    hex::encode(&hash[..16])
}

/// 生成校验 hash：`SHA256(machine_id + seed + license_code)`
fn compute_activation_hash(machine_id: &str, code: &str) -> String {
    let input = format!("{}-{}-{}", machine_id, LICENSE_SEED, code);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..12])
}

/// 校验授权码是否有效
fn validate_license_code(machine_id: &str, code: &str, hash: &str) -> bool {
    let expected = compute_activation_hash(machine_id, code);
    expected == hash
}

/// 授权码格式：`machine_hash-activation_hash`（12位短 hash）
/// 例如：`a1b2c3d4-e5f678901234`
fn parse_license_code(code: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = code.trim().split('-').collect();
    if parts.len() < 2 {
        return None;
    }
    // 最后一段是 activation hash
    let hash_part = parts.last()?;
    let machine_part = &code[..code.len() - hash_part.len() - 1];
    Some((machine_part, hash_part))
}

/// 保存许可证到本地
fn save_license(info: &LicenseInfo) -> Result<(), String> {
    let path = license_path().ok_or("无法获取许可证存储路径")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let json = serde_json::to_string(info).map_err(|e| format!("序列化失败: {}", e))?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, json.as_bytes());
    std::fs::write(&path, &encoded).map_err(|e| format!("保存失败: {}", e))?;
    Ok(())
}

/// 加载已保存的许可证
fn load_license() -> Option<LicenseInfo> {
    let path = license_path()?;
    let encoded = std::fs::read_to_string(&path).ok()?;
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded.trim()).ok()?;
    let json = String::from_utf8(decoded).ok()?;
    serde_json::from_str::<LicenseInfo>(&json).ok()
}

/// 检查是否已激活
pub fn is_activated() -> bool {
    LICENSE_ACTIVATED.get().copied().unwrap_or(false)
}

/// 生成授权码（只有开发者需要调用）
/// 
/// 用法：调用 `generate_license(machine_id)` 得到授权码，发给用户
pub fn generate_license(machine_id: &str) -> String {
    let hash = compute_activation_hash(machine_id, machine_id);
    format!("{}-{}", machine_id, hash)
}

/// 启动时检查许可证
pub fn check_license_on_startup() {
    if let Some(saved) = load_license() {
        let current_id = get_machine_id();
        if saved.machine_id == current_id {
            // 从 license_code 中提取 machine_hash 和 stored_hash
            if let Some((machine_hash, stored_hash)) = saved.license_code.rsplit_once('-') {
                let expected_hash = compute_activation_hash(&current_id, machine_hash);
                if stored_hash == expected_hash {
                    let _ = LICENSE_ACTIVATED.set(true);
                    logger::log_info(&format!(
                        "[License] 已激活, machine={}",
                        &current_id[..8]
                    ));
                    return;
                }
            }
        }
        logger::log_warn("[License] 本地许可证无效，需要重新激活");
    }

    let machine_id = get_machine_id();
    logger::log_info(&format!(
        "[License] 未激活，machine_id={}",
        &machine_id[..8]
    ));

    // 通知前端显示激活界面
    if let Some(app) = get_app_handle() {
        let _ = app.emit("license:required", serde_json::json!({
            "machineId": &machine_id[..8]
        }));
    }
}

/// 前端提交的激活请求
#[tauri::command]
pub fn activate_license(license_code: String) -> Result<String, String> {
    let machine_id = get_machine_id();
    let code = license_code.trim();

    // 解析授权码
    let (machine_hash, _activation_hash) = parse_license_code(code)
        .ok_or_else(|| "授权码格式不正确".to_string())?;

    // 验证：用全局 seed 重新计算 hash
    let expected = compute_activation_hash(&machine_id, machine_hash);
    let full_code = format!("{}-{}", machine_hash, expected);

    if full_code != code {
        return Err("授权码无效".to_string());
    }

    // 保存
    let info = LicenseInfo {
        machine_id: machine_id.clone(),
        license_code: code.to_string(),
        activated_at: chrono::Utc::now().timestamp(),
    };
    save_license(&info)?;
    let _ = LICENSE_ACTIVATED.set(true);

    logger::log_info(&format!(
        "[License] 激活成功, machine={}",
        &machine_id[..8]
    ));
    Ok("激活成功".to_string())
}

/// 生成特定机器的授权码（命令行/手动用）
#[tauri::command]
pub fn generate_license_code(target_machine_id: String) -> String {
    generate_license(&target_machine_id)
}

/// 获取本机完整 machine_id（用于给别人生成授权码时提供）
#[tauri::command]
pub fn get_full_machine_id() -> String {
    get_machine_id()
}

/// 获取当前激活状态
#[tauri::command]
pub fn get_license_status() -> serde_json::Value {
    let machine_id = get_machine_id();
    if let Some(info) = load_license() {
        if info.machine_id == machine_id {
            if let Some((machine_hash, stored_hash)) = info.license_code.rsplit_once('-') {
                let expected = compute_activation_hash(&machine_id, machine_hash);
                let valid = stored_hash == expected;
                return serde_json::json!({
                    "activated": valid,
                    "machineId": &machine_id[..8],
                    "activatedAt": info.activated_at,
                });
            }
        }
    }
    serde_json::json!({
        "activated": false,
        "machineId": &machine_id[..8],
    })
}
