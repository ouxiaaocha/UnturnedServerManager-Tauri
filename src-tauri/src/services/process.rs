use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use chrono::Local;

use crate::models::config::ServerProfile;
use crate::models::state::ServerState;
use crate::services::log_service::LogService;

/// 输出缓冲区最大保留行数，超过后按批次裁剪
const OUTPUT_RETAIN_LINES: usize = 1000;
/// 每次裁剪的行数
const OUTPUT_TRIM_BATCH: usize = 100;
/// 定期维护时保留的行数（更激进的压缩）
const MAINTENANCE_RETAIN_LINES: usize = 500;
/// 定期维护间隔（6 小时），用于压缩输出缓冲区内存占用
const MAINTENANCE_INTERVAL_SECS: u64 = 6 * 60 * 60;

struct OutputBuffer {
    lines: Vec<String>,
    start_index: usize,
}

impl OutputBuffer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            start_index: 0,
        }
    }

    fn push(&mut self, line: String) {
        if self.lines.len() >= OUTPUT_RETAIN_LINES {
            let remove_count = OUTPUT_TRIM_BATCH.min(self.lines.len());
            self.lines.drain(0..remove_count);
            self.start_index += remove_count;
        }
        self.lines.push(line);
    }

    fn reset_with(&mut self, line: String) {
        self.lines.clear();
        self.lines.shrink_to_fit();
        self.start_index = 0;
        self.lines.push(line);
    }

    fn compact(&mut self, retain_lines: usize) -> (usize, usize, usize) {
        let before_len = self.lines.len();
        let before_capacity = self.lines.capacity();
        if before_len > retain_lines {
            let remove_count = before_len - retain_lines;
            self.lines.drain(0..remove_count);
            self.start_index += remove_count;
        }
        if self.lines.capacity() > self.lines.len().saturating_mul(2).max(retain_lines) {
            self.lines.shrink_to_fit();
        }
        (before_len, self.lines.len(), before_capacity)
    }

    fn total_count(&self) -> usize {
        self.start_index + self.lines.len()
    }

    fn new_lines(&self, from_index: usize) -> Vec<String> {
        if self.lines.is_empty() {
            return vec![];
        }

        if from_index <= self.start_index {
            return self.lines.clone();
        }

        let offset = from_index - self.start_index;
        if offset < self.lines.len() {
            self.lines[offset..].to_vec()
        } else {
            vec![]
        }
    }
}

fn push_output_line(output_buffer: &Arc<Mutex<OutputBuffer>>, line: String) {
    let mut buffer = output_buffer.lock().unwrap_or_else(|e| e.into_inner());
    buffer.push(line);
}

struct ServerProcessSession {
    child: Option<Child>,
    state: ServerState,
    start_time: Option<Instant>,
    output_buffer: Arc<Mutex<OutputBuffer>>,
    launch_mode: String,
}

impl ServerProcessSession {
    fn new() -> Self {
        Self {
            child: None,
            state: ServerState::Stopped,
            start_time: None,
            output_buffer: Arc::new(Mutex::new(OutputBuffer::new())),
            launch_mode: "internet".to_string(),
        }
    }

    fn is_active(&self) -> bool {
        matches!(
            self.state,
            ServerState::Starting
                | ServerState::Running
                | ServerState::Restarting
                | ServerState::Updating
        )
    }
}

/// 管理多个 Unturned 存档实例的生命周期、输出缓冲和游戏日志
pub struct ProcessManager {
    sessions: HashMap<String, ServerProcessSession>,
    logs_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(logs_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(logs_dir.join("game"));
        Self {
            sessions: HashMap::new(),
            logs_dir,
        }
    }

    pub fn state(&self) -> ServerState {
        if self.sessions.values().any(ServerProcessSession::is_active) {
            ServerState::Running
        } else {
            ServerState::Stopped
        }
    }

    pub fn pid(&self) -> Option<u32> {
        self.sessions.values().find_map(|session| {
            if session.is_active() {
                session.child.as_ref().map(|c| c.id())
            } else {
                None
            }
        })
    }

    pub fn uptime_secs(&self) -> u64 {
        self.sessions
            .values()
            .filter_map(|session| session.start_time.map(|t| t.elapsed().as_secs()))
            .max()
            .unwrap_or(0)
    }

    pub fn state_for(&self, save_id: &str) -> ServerState {
        self.sessions
            .get(save_id)
            .map(|session| session.state)
            .unwrap_or(ServerState::Stopped)
    }

    pub fn pid_for(&self, save_id: &str) -> Option<u32> {
        self.sessions
            .get(save_id)
            .and_then(|session| session.child.as_ref().map(|c| c.id()))
    }

    pub fn uptime_secs_for(&self, save_id: &str) -> u64 {
        self.sessions
            .get(save_id)
            .and_then(|session| session.start_time.map(|t| t.elapsed().as_secs()))
            .unwrap_or(0)
    }

    pub fn launch_mode_for(&self, save_id: &str) -> String {
        self.sessions
            .get(save_id)
            .map(|session| session.launch_mode.clone())
            .unwrap_or_else(|| "internet".to_string())
    }

    pub fn running_save_ids(&mut self) -> Vec<String> {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        ids.into_iter()
            .filter(|save_id| self.is_running_for(save_id))
            .collect()
    }

    pub fn running_count(&mut self) -> usize {
        self.running_save_ids().len()
    }

    pub fn start(
        &mut self,
        save_id: &str,
        launch_mode: &str,
        profile: &ServerProfile,
    ) -> Result<(), String> {
        if self.is_running_for(save_id) || matches!(self.state_for(save_id), ServerState::Starting)
        {
            return Err(format!("存档 {} 的服务器已在运行", save_id));
        }

        let exe_path = Path::new(&profile.server_root).join("Unturned.exe");
        if !exe_path.exists() {
            return Err(format!("找不到 Unturned.exe: {}", exe_path.display()));
        }

        let session = self
            .sessions
            .entry(save_id.to_string())
            .or_insert_with(ServerProcessSession::new);
        session.state = ServerState::Starting;
        session.launch_mode = launch_mode.to_string();

        {
            let mut buffer = session
                .output_buffer
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            buffer.reset_with("[系统] 正在启动服务器...".to_string());
        }

        let mut cmd = Command::new(&exe_path);
        cmd.args(["-batchmode", "-nographics", &profile.server_entry])
            .current_dir(&profile.server_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Clear sensitive environment variables for child process
        cmd.env_remove("RCON_PASSWORD");
        cmd.env_remove("STEAM_PASSWORD");
        cmd.env_remove("API_KEY");
        cmd.env_remove("SECRET_KEY");
        cmd.env_remove("AWS_SECRET_ACCESS_KEY");

        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let child = cmd.spawn().map_err(|e| format!("启动失败: {}", e))?;

        // Store child and set state BEFORE spawning reader threads
        session.child = Some(child);
        session.state = ServerState::Running;
        session.start_time = Some(Instant::now());

        let child_ref = session.child.as_mut().unwrap();

        if let Some(stdout) = child_ref.stdout.take() {
            let output_buffer = Arc::clone(&session.output_buffer);
            let game_log_dir = self.logs_dir.join("game");
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                let mut log_writer = GameLogWriter::new(game_log_dir);
                for line in reader.lines().map_while(Result::ok) {
                    push_output_line(&output_buffer, line.clone());
                    let _ = log_writer.append(&line);
                }
            });
        }

        if let Some(stderr) = child_ref.stderr.take() {
            let output_buffer = Arc::clone(&session.output_buffer);
            let game_log_dir = self.logs_dir.join("game");
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                let mut log_writer = GameLogWriter::new(game_log_dir);
                for line in reader.lines().map_while(Result::ok) {
                    let formatted = format!("[ERROR] {}", line);
                    push_output_line(&output_buffer, formatted.clone());
                    let _ = log_writer.append(&formatted);
                }
            });
        }

        push_output_line(&session.output_buffer, "[系统] 服务器已启动".to_string());

        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        ids.into_iter().any(|save_id| self.is_running_for(&save_id))
    }

    pub fn is_running_for(&mut self, save_id: &str) -> bool {
        let Some(session) = self.sessions.get_mut(save_id) else {
            return false;
        };

        if let Some(child) = &mut session.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    push_output_line(
                        &session.output_buffer,
                        "[系统] 服务器进程已退出".to_string(),
                    );
                    session.state = ServerState::Stopped;
                    session.child = None;
                    session.start_time = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn force_stop_for(&mut self, save_id: &str) -> Result<(), String> {
        let session = self
            .sessions
            .entry(save_id.to_string())
            .or_insert_with(ServerProcessSession::new);

        if let Some(mut child) = session.child.take() {
            session.state = ServerState::Stopping;
            child.kill().map_err(|e| format!("强制停止失败: {}", e))?;
            let _ = child.wait();
            push_output_line(
                &session.output_buffer,
                "[系统] 服务器已强制停止".to_string(),
            );
        }
        session.state = ServerState::Stopped;
        session.start_time = None;
        Ok(())
    }

    pub fn record_sent_command_for(&self, save_id: &str, command: &str) {
        if let Some(session) = self.sessions.get(save_id) {
            push_output_line(&session.output_buffer, format!("[命令] > {}", command));
        }
    }

    pub fn record_system_message_for(&self, save_id: &str, message: &str) {
        if let Some(session) = self.sessions.get(save_id) {
            push_output_line(&session.output_buffer, format!("[系统] {}", message));
        }
    }

    pub fn get_new_output(&self, from_index: usize) -> Vec<String> {
        self.sessions
            .values()
            .next()
            .map(|session| {
                let buffer = session
                    .output_buffer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                buffer.new_lines(from_index)
            })
            .unwrap_or_default()
    }

    pub fn get_new_output_for(&self, save_id: &str, from_index: usize) -> Vec<String> {
        self.sessions
            .get(save_id)
            .map(|session| {
                let buffer = session
                    .output_buffer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                buffer.new_lines(from_index)
            })
            .unwrap_or_default()
    }

    pub fn output_count(&self) -> usize {
        self.sessions
            .values()
            .map(|session| {
                session
                    .output_buffer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .total_count()
            })
            .max()
            .unwrap_or(0)
    }

    pub fn output_count_for(&self, save_id: &str) -> usize {
        self.sessions
            .get(save_id)
            .map(|session| {
                session
                    .output_buffer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .total_count()
            })
            .unwrap_or(0)
    }

    pub fn compact_output_cache(&mut self, retain_lines: usize) -> (usize, usize, usize) {
        let mut before_len = 0;
        let mut after_len = 0;
        let mut before_capacity = 0;
        for session in self.sessions.values_mut() {
            let mut buffer = session
                .output_buffer
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let (before, after, capacity) = buffer.compact(retain_lines);
            before_len += before;
            after_len += after;
            before_capacity += capacity;
        }
        (before_len, after_len, before_capacity)
    }
}

/// Unturned 服务器允许的命令白名单
/// 来源: https://github.com/SmartlyDressedGames/Unturned-Docs/blob/master/ServerHosting.md
const ALLOWED_COMMANDS: &[&str] = &[
    // 玩家管理
    "admin",
    "unadmin",
    "kick",
    "ban",
    "unban",
    "permit",
    "unpermit",
    // 物品和传送
    "give",
    "vehicle",
    "teleport",
    "experience",
    // 服务器管理
    "save",
    "shutdown",
    "slay",
    "spy",
    "filter",
    "mode",
    "name",
    "password",
    "port",
    "sync",
    "timeout",
    "queue",
    "chatrate",
    "maxplayers",
    "loadout",
    // 信息和通知
    "say",
    "announce",
    "welcome",
    "decay",
    // 时间和天气
    "time",
    "day",
    "night",
    "weather",
    "storm",
    "airdrop",
    // 调试
    "debug",
    "cheats",
    "cycle",
    "gold",
    "resetconfig",
    // Rocket 插件命令（如果安装）
    "rocket",
    "reload",
    "unload",
    "load",
];

pub fn normalize_server_command(command: &str) -> Result<&str, String> {
    // 检查换行符
    if command.contains('\n') || command.contains('\r') {
        return Err("命令不能包含换行".to_string());
    }

    let command = command.trim();
    if command.is_empty() {
        return Err("命令不能为空".to_string());
    }

    // 检查危险字符（命令注入防护）
    if command.contains(';') || command.contains('&') || command.contains('|') {
        return Err("命令包含非法字符（; & |）".to_string());
    }

    // 提取命令名（第一个单词）
    let command_name = command
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    // 白名单验证
    if !ALLOWED_COMMANDS.contains(&command_name.as_str()) {
        return Err(format!(
            "不允许的命令: '{}'. 允许的命令: {}",
            command_name,
            ALLOWED_COMMANDS.join(", ")
        ));
    }

    Ok(command)
}

#[cfg(test)]
mod command_validation_tests {
    use super::*;

    #[test]
    fn test_valid_commands() {
        assert!(normalize_server_command("save").is_ok());
        assert!(normalize_server_command("  save  ").is_ok());
        assert!(normalize_server_command("give 76561198000000000 122 1").is_ok());
        assert!(normalize_server_command("SAY Hello World").is_ok());
    }

    #[test]
    fn test_invalid_commands() {
        assert!(normalize_server_command("").is_err());
        assert!(normalize_server_command("   ").is_err());
        assert!(normalize_server_command("save\nshutdown").is_err());
        assert!(normalize_server_command("save; shutdown").is_err());
        assert!(normalize_server_command("save && shutdown").is_err());
        assert!(normalize_server_command("save | grep").is_err());
        assert!(normalize_server_command("unknown_command").is_err());
    }
}

pub fn start_output_cache_maintenance(
    process: Arc<Mutex<ProcessManager>>,
    log: Arc<Mutex<LogService>>,
) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(MAINTENANCE_INTERVAL_SECS));

        let (before, after, before_capacity, state, pid) = {
            let mut pm = process.lock().unwrap_or_else(|e| e.into_inner());
            let state = pm.state().to_string();
            let pid = pm.pid();
            let (before, after, before_capacity) =
                pm.compact_output_cache(MAINTENANCE_RETAIN_LINES);
            (before, after, before_capacity, state, pid)
        };

        let ls = log.lock().unwrap_or_else(|e| e.into_inner());
        ls.log_app(&format!(
            "[维护] 输出缓存检查: 状态={}, PID={:?}, 行数 {}->{}, 原容量={}",
            state, pid, before, after, before_capacity
        ));
    });
}

struct GameLogWriter {
    game_log_dir: PathBuf,
    current_date: String,
    file: Option<File>,
    lines_since_flush: u32,
}

const FLUSH_INTERVAL: u32 = 20;

impl GameLogWriter {
    fn new(game_log_dir: PathBuf) -> Self {
        Self {
            game_log_dir,
            current_date: String::new(),
            file: None,
            lines_since_flush: 0,
        }
    }

    fn append(&mut self, line: &str) -> std::io::Result<()> {
        let now = Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let time = now.format("%H:%M:%S").to_string();

        if self.file.is_none() || self.current_date != date {
            // 切换日期前先 flush 旧文件
            if let Some(file) = self.file.as_mut() {
                let _ = file.flush();
            }
            let file_path = self.game_log_dir.join(format!("{}.log", date));
            self.file = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?,
            );
            self.current_date = date;
            self.lines_since_flush = 0;
        }

        if let Some(file) = self.file.as_mut() {
            writeln!(file, "[{}] {}", time, line)?;
            self.lines_since_flush += 1;
            if self.lines_since_flush >= FLUSH_INTERVAL {
                file.flush()?;
                self.lines_since_flush = 0;
            }
        }
        Ok(())
    }
}

impl Drop for GameLogWriter {
    fn drop(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let _ = file.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_buffer_uses_monotonic_total_count_after_trimming() {
        let mut buffer = OutputBuffer::new();
        for i in 0..=OUTPUT_RETAIN_LINES {
            buffer.push(format!("line {}", i));
        }

        assert!(buffer.start_index > 0);
        assert_eq!(buffer.total_count(), OUTPUT_RETAIN_LINES + 1);
        assert_eq!(
            buffer.lines.len(),
            OUTPUT_RETAIN_LINES + 1 - OUTPUT_TRIM_BATCH
        );
    }

    #[test]
    fn output_buffer_returns_retained_lines_when_requested_index_was_pruned() {
        let mut buffer = OutputBuffer::new();
        for i in 0..=OUTPUT_RETAIN_LINES {
            buffer.push(format!("line {}", i));
        }

        let lines = buffer.new_lines(0);

        assert_eq!(lines.first().unwrap(), "line 100");
        assert_eq!(lines.last().unwrap(), "line 1000");
    }

    #[test]
    fn validate_server_command_rejects_empty_input() {
        assert_eq!(normalize_server_command("  ").unwrap_err(), "命令不能为空");
    }

    #[test]
    fn validate_server_command_rejects_multiline_input() {
        assert_eq!(
            normalize_server_command("Save\nShutdown").unwrap_err(),
            "命令不能包含换行"
        );
        assert_eq!(
            normalize_server_command("Save\rShutdown").unwrap_err(),
            "命令不能包含换行"
        );
    }
}
