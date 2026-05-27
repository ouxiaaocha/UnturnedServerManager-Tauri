use std::fs::{self, OpenOptions};
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

pub struct ProcessManager {
    child: Option<Child>,
    state: ServerState,
    start_time: Option<Instant>,
    output_lines: Arc<Mutex<Vec<String>>>,
    logs_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(logs_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(logs_dir.join("game"));
        Self {
            child: None,
            state: ServerState::Stopped,
            start_time: None,
            output_lines: Arc::new(Mutex::new(Vec::new())),
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
        self.start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
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
            let mut lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
            lines.clear();
            lines.push("[系统] 正在启动服务器...".to_string());
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

        let child = cmd.spawn()
            .map_err(|e| format!("启动失败: {}", e))?;

        // Store child and set state BEFORE spawning reader threads
        self.child = Some(child);
        self.state = ServerState::Running;
        self.start_time = Some(Instant::now());

        let child_ref = self.child.as_mut().unwrap();

        if let Some(stdout) = child_ref.stdout.take() {
            let lines_clone = Arc::clone(&self.output_lines);
            let game_log_dir = self.logs_dir.join("game");
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    {
                        let mut lines = lines_clone.lock().unwrap_or_else(|e| e.into_inner());
                        if lines.len() > 1000 {
                            lines.drain(0..100);
                        }
                        lines.push(line.clone());
                    }
                    let _ = append_game_log(&game_log_dir, &line);
                }
            });
        }

        if let Some(stderr) = child_ref.stderr.take() {
            let lines_clone = Arc::clone(&self.output_lines);
            let game_log_dir = self.logs_dir.join("game");
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let formatted = format!("[ERROR] {}", line);
                    {
                        let mut lines = lines_clone.lock().unwrap_or_else(|e| e.into_inner());
                        lines.push(formatted.clone());
                    }
                    let _ = append_game_log(&game_log_dir, &formatted);
                }
            });
        }

        {
            let mut lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
            lines.push("[系统] 服务器已启动".to_string());
        }

        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let mut lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
                    lines.push("[系统] 服务器进程已退出".to_string());
                    drop(lines);
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
            let mut lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
            lines.push("[系统] 服务器已强制停止".to_string());
        }
        self.state = ServerState::Stopped;
        self.start_time = None;
        Ok(())
    }

    pub fn mark_stopped(&mut self) {
        self.child = None;
        self.state = ServerState::Stopped;
        self.start_time = None;
        let mut lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
        lines.push("[系统] 服务器已停止".to_string());
    }

    pub fn get_new_output(&self, from_index: usize) -> Vec<String> {
        let lines = self.output_lines.lock().unwrap_or_else(|e| e.into_inner());
        if from_index < lines.len() {
            lines[from_index..].to_vec()
        } else {
            vec![]
        }
    }

    pub fn output_count(&self) -> usize {
        self.output_lines.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

fn append_game_log(game_log_dir: &Path, line: &str) -> std::io::Result<()> {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:%M:%S").to_string();

    let file_path = game_log_dir.join(format!("{}.log", date));
    let mut file = OpenOptions::new().create(true).append(true).open(file_path)?;
    writeln!(file, "[{}] {}", time, line)
}
