use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager};

// ─── Data Structures ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Settings {
    game_path: String,
    wegame_path: String,
    store_path: String,
    ask_before_update: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let store = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RocomHelper");
        Settings {
            game_path: String::new(),
            wegame_path: String::new(),
            store_path: store.to_string_lossy().to_string(),
            ask_before_update: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Serialize, Clone)]
struct VersionInfo {
    latest: Option<String>,
    local: Option<String>,
}

#[derive(Serialize, Clone)]
struct LogPayload {
    level: String,
    msg: String,
}

#[derive(Serialize, Clone)]
struct DownloadProgress {
    downloaded: u64,
    total: u64,
}

// ─── Helper Functions ─────────────────────────────────────────

fn get_settings_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".rocom-helper-launcher")
    });
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn get_store_dir(settings: &Settings) -> PathBuf {
    let p = PathBuf::from(&settings.store_path);
    let _ = fs::create_dir_all(&p);
    p
}

fn get_version_file(store_dir: &PathBuf) -> PathBuf {
    store_dir.join("version.txt")
}

fn get_local_version(store_dir: &PathBuf) -> Option<String> {
    let vf = get_version_file(store_dir);
    fs::read_to_string(&vf)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn find_helper_exe(store_dir: &PathBuf) -> Option<PathBuf> {
    // Look for roco_helper*.exe in the store directory
    if let Ok(entries) = fs::read_dir(store_dir) {
        let mut exes: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|ext| ext.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
            })
            .filter(|p| {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                name.contains("roco") || name.contains("helper")
            })
            .collect();
        exes.sort_by(|a, b| {
            b.metadata()
                .map(|m| m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .map(|m| m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });
        exes.first().cloned()
    } else {
        None
    }
}

fn log_to_frontend(app: &AppHandle, level: &str, msg: &str) {
    let _ = app.emit(
        "backend-log",
        LogPayload {
            level: level.to_string(),
            msg: msg.to_string(),
        },
    );
}

fn load_settings_from_file(app: &AppHandle) -> Settings {
    let path = get_settings_path(app);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

async fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let client = reqwest::Client::builder()
        .user_agent("RocomHelper-Launcher/1.0")
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .get("https://api.github.com/repos/h3110w0r1d-y/rocom-helper/releases/latest")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("请求GitHub API失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API返回错误状态: {}", resp.status()));
    }

    resp.json::<GitHubRelease>()
        .await
        .map_err(|e| format!("解析GitHub响应失败: {}", e))
}

// ─── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
async fn check_version(app: AppHandle) -> Result<VersionInfo, String> {
    let settings = load_settings_from_file(&app);
    let store_dir = get_store_dir(&settings);

    let local = get_local_version(&store_dir);

    let latest = match fetch_latest_release().await {
        Ok(release) => {
            log_to_frontend(
                &app,
                "info",
                &format!("GitHub最新版本: {}", release.tag_name),
            );
            Some(release.tag_name)
        }
        Err(e) => {
            log_to_frontend(&app, "error", &format!("获取最新版本失败: {}", e));
            None
        }
    };

    Ok(VersionInfo { latest, local })
}

#[tauri::command]
async fn download_helper(app: AppHandle) -> Result<(), String> {
    let settings = load_settings_from_file(&app);
    let store_dir = get_store_dir(&settings);

    log_to_frontend(&app, "info", "正在获取版本信息...");
    let release = fetch_latest_release()
        .await
        .map_err(|e| {
            log_to_frontend(&app, "error", &format!("获取版本信息失败: {}", e));
            e
        })?;

    let version = release.tag_name.clone();

    // Find the main .exe asset
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".exe"))
        .or_else(|| release.assets.first())
        .ok_or_else(|| {
            let msg = "未找到可下载的文件".to_string();
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

    let download_url = asset.browser_download_url.clone();
    let file_name = asset.name.clone();
    let total_size = asset.size;
    let dest_path = store_dir.join(&file_name);

    log_to_frontend(
        &app,
        "info",
        &format!("开始下载 {} ({:.1} MB)...", file_name, total_size as f64 / 1_048_576.0),
    );

    // Download with progress
    let client = reqwest::Client::builder()
        .user_agent("RocomHelper-Launcher/1.0")
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("下载请求失败: {}", e);
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

    if !resp.status().is_success() {
        let msg = format!("下载失败，HTTP状态: {}", resp.status());
        log_to_frontend(&app, "error", &msg);
        return Err(msg);
    }

    // Stream the download to file
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&dest_path)
        .await
        .map_err(|e| {
            let msg = format!("创建文件失败: {}", e);
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

    let mut downloaded: u64 = 0;
    let mut last_report = 0u64;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            let msg = format!("下载数据块失败: {}", e);
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {}", e))?;

        downloaded += chunk.len() as u64;

        // Report progress every 1MB
        if downloaded - last_report >= 1_048_576 || downloaded == total_size {
            let percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0) as u32
            } else {
                0
            };
            log_to_frontend(
                &app,
                "info",
                &format!("下载进度: {}% ({:.1} MB)", percent, downloaded as f64 / 1_048_576.0),
            );
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    downloaded,
                    total: total_size,
                },
            );
            last_report = downloaded;
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("文件写入完成失败: {}", e))?;

    // Write version file
    let version_path = get_version_file(&store_dir);
    fs::write(&version_path, &version).map_err(|e| format!("写入版本文件失败: {}", e))?;

    // Clean up old exe files (keep only the latest)
    if let Ok(entries) = fs::read_dir(&store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false)
                && path != dest_path
            {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if name.to_lowercase().contains("roco") || name.to_lowercase().contains("helper") {
                    let _ = fs::remove_file(&path);
                    log_to_frontend(&app, "info", &format!("清理旧文件: {}", name));
                }
            }
        }
    }

    log_to_frontend(
        &app,
        "success",
        &format!("下载完成: {} ({})", file_name, version),
    );

    Ok(())
}

#[tauri::command]
async fn launch_rocom(app: AppHandle) -> Result<(), String> {
    let settings = load_settings_from_file(&app);
    let store_dir = get_store_dir(&settings);

    let exe = find_helper_exe(&store_dir).ok_or_else(|| {
        let msg = "未找到 Rocom Helper 可执行文件，请先下载".to_string();
        log_to_frontend(&app, "error", &msg);
        msg
    })?;

    log_to_frontend(
        &app,
        "info",
        &format!("启动 Rocom Helper: {}", exe.display()),
    );

    Command::new(&exe)
        .spawn()
        .map_err(|e| {
            let msg = format!("启动 Rocom Helper 失败: {}", e);
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

    log_to_frontend(&app, "success", "Rocom Helper 已启动");
    Ok(())
}

#[tauri::command]
async fn launch_game(app: AppHandle, game_type: String) -> Result<(), String> {
    let settings = load_settings_from_file(&app);

    let path = match game_type.as_str() {
        "game" => &settings.game_path,
        "wegame" => &settings.wegame_path,
        _ => return Err(format!("未知的启动类型: {}", game_type)),
    };

    if path.is_empty() {
        let msg = format!("{} 路径未设置", game_type);
        log_to_frontend(&app, "warning", &msg);
        return Err(msg);
    }

    let exe_path = PathBuf::from(path);
    if !exe_path.exists() {
        let msg = format!("{} 路径不存在: {}", game_type, path);
        log_to_frontend(&app, "error", &msg);
        return Err(msg);
    }

    log_to_frontend(&app, "info", &format!("启动 {}: {}", game_type, path));

    Command::new(&exe_path)
        .spawn()
        .map_err(|e| {
            let msg = format!("启动 {} 失败: {}", game_type, e);
            log_to_frontend(&app, "error", &msg);
            msg
        })?;

    log_to_frontend(&app, "success", &format!("{} 已启动", game_type));
    Ok(())
}

#[tauri::command]
async fn test_github_connectivity(app: AppHandle) -> Result<bool, String> {
    log_to_frontend(&app, "info", "正在测试 GitHub 连通性...");
    let client = reqwest::Client::builder()
        .user_agent("RocomHelper-Launcher/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    match client
        .get("https://api.github.com")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                log_to_frontend(&app, "success", "GitHub 连通性测试通过");
                Ok(true)
            } else {
                log_to_frontend(
                    &app,
                    "warning",
                    &format!("GitHub 返回非成功状态: {}", resp.status()),
                );
                Ok(false)
            }
        }
        Err(e) => {
            log_to_frontend(&app, "error", &format!("GitHub 连接失败: {}", e));
            Ok(false)
        }
    }
}

#[tauri::command]
fn delete_helper(app: AppHandle) -> Result<(), String> {
    let settings = load_settings_from_file(&app);
    let store_dir = get_store_dir(&settings);

    let mut deleted_any = false;

    // Delete exe files
    if let Ok(entries) = fs::read_dir(&store_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if name.to_lowercase().contains("roco") || name.to_lowercase().contains("helper") {
                    match fs::remove_file(&path) {
                        Ok(_) => {
                            log_to_frontend(&app, "info", &format!("已删除: {}", name));
                            deleted_any = true;
                        }
                        Err(e) => {
                            log_to_frontend(&app, "error", &format!("删除 {} 失败: {}", name, e));
                        }
                    }
                }
            }
        }
    }

    // Delete version file
    let version_path = get_version_file(&store_dir);
    if version_path.exists() {
        let _ = fs::remove_file(&version_path);
        log_to_frontend(&app, "info", "已删除版本记录文件");
        deleted_any = true;
    }

    if !deleted_any {
        log_to_frontend(&app, "warning", "未找到需要删除的文件");
    } else {
        log_to_frontend(&app, "success", "本地 Rocom Helper 已清除");
    }

    Ok(())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(load_settings_from_file(&app))
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = get_settings_path(&app);
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入设置文件失败: {}", e))?;
    Ok(())
}

// ─── App Entry Point ──────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_version,
            download_helper,
            delete_helper,
            launch_rocom,
            launch_game,
            test_github_connectivity,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
