use serde::{Deserialize, Serialize};

/// GitHub Release 更新信息
#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    /// 最新版本号（不含 v 前缀）
    pub latest_version: String,
    /// 当前版本号
    pub current_version: String,
    /// 是否有可用更新
    pub has_update: bool,
    /// 更新日志内容（Markdown 格式）
    pub body: String,
    /// GitHub Release 页面链接
    pub html_url: String,
    /// 发布时间（ISO 8601 格式）
    pub published_at: String,
}

/// 检查 GitHub Releases 获取最新版本信息
///
/// 通过 GitHub API 查询最新的 Release，对比版本号判断是否有更新。
/// 网络超时或请求失败时返回 Err，前端显示"检测更新失败"。
#[tauri::command]
pub fn check_for_updates() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");
    let url = "https://api.github.com/repos/ouxiaaocha/UnturnedServerManager-Tauri/releases/latest";

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("UnturnedServerManager")
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let status = resp.status();
    let body_text = resp
        .text()
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        // 403 / 429 通常是 GitHub API 速率限制
        if status.as_u16() == 403 || status.as_u16() == 429 {
            // 尝试从响应中提取限流重置时间
            let reset_hint = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .and_then(|v| v["message"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "请求过于频繁，请稍后再试".to_string());
            return Err(format!("GitHub API 速率限制: {}", reset_hint));
        }
        return Err(format!("GitHub API 返回错误: HTTP {}", status));
    }

    let json: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let tag = json["tag_name"].as_str().unwrap_or("");
    let latest = tag.trim_start_matches('v');

    let body = json["body"]
        .as_str()
        .unwrap_or("无更新日志")
        .to_string();

    let html_url = json["html_url"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let published_at = json["published_at"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(UpdateInfo {
        latest_version: latest.to_string(),
        current_version: current.to_string(),
        has_update: latest != current,
        body,
        html_url,
        published_at,
    })
}
