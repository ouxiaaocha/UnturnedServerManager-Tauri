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

/// 管理 Unturned 服务端进程的生命周期、输出缓冲和游戏日志
pub struct ProcessManager {
    child: Option<Child>,
    state: ServerState,
    start_time: Option<Instant>,
    output_buffer: Arc<Mutex<OutputBuffer>>,
    logs_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(logs_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(logs_dir.join("game"));
        Self {
            child: None,
            state: ServerState::Stopped,
            start_time: None,
            output_buffer: Arc::new(Mutex::new(OutputBuffer::new())),
            logs_dir,
        }
    }

    pub fn state(&self) -> ServerState {
        self.state
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(|c| c.id())
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }

    pub fn start(&mut self, profile: &ServerProfile) -> Result<(), String> {
        if self.state == ServerState::Running || self.state == ServerState::Starting {
            return Err("服务器已在运行".to_string());
        }

        let exe_path = Path::new(&profile.server_root).join("Unturned.exe");
        if !exe_path.exists() {
            return Err(format!("找不到 Unturned.exe: {}", exe_path.display()));
        }

        self.state = ServerState::Starting;

        {
            let mut buffer = self.output_buffer.lock().unwrap_or_else(|e| e.into_inner());
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
        self.child = Some(child);
        self.state = ServerState::Running;
        self.start_time = Some(Instant::now());

        let child_ref = self.child.as_mut().unwrap();

        if let Some(stdout) = child_ref.stdout.take() {
            let output_buffer = Arc::clone(&self.output_buffer);
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
            let output_buffer = Arc::clone(&self.output_buffer);
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

        push_output_line(&self.output_buffer, "[系统] 服务器已启动".to_string());

        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    push_output_line(&self.output_buffer, "[系统] 服务器进程已退出".to_string());
                    self.state = ServerState::Stopped;
                    self.child = None;
                    self.start_time = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn force_stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            self.state = ServerState::Stopping;
            child.kill().map_err(|e| format!("强制停止失败: {}", e))?;
            let _ = child.wait();
            push_output_line(&self.output_buffer, "[系统] 服务器已强制停止".to_string());
        }
        self.state = ServerState::Stopped;
        self.start_time = None;
        Ok(())
    }

    pub fn record_sent_command(&self, command: &str) {
        push_output_line(&self.output_buffer, format!("[命令] > {}", command));
    }

    pub fn record_system_message(&self, message: &str) {
        push_output_line(&self.output_buffer, format!("[系统] {}", message));
    }

    pub fn get_new_output(&self, from_index: usize) -> Vec<String> {
        let buffer = self.output_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.new_lines(from_index)
    }

    pub fn output_count(&self) -> usize {
        self.output_buffer
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .total_count()
    }

    pub fn compact_output_cache(&mut self, retain_lines: usize) -> (usize, usize, usize) {
        let mut buffer = self.output_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.compact(retain_lines)
    }
}

pub fn normalize_server_command(command: &str) -> Result<&str, String> {
    if command.contains('\n') || command.contains('\r') {
        return Err("命令不能包含换行".to_string());
    }
    let command = command.trim();
    if command.is_empty() {
        return Err("命令不能为空".to_string());
    }
    Ok(command)
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
