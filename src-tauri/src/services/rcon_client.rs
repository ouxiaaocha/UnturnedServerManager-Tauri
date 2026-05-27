use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct RconClient {
    stream: Option<TcpStream>,
    responses: Arc<Mutex<Vec<String>>>,
    reader_alive: Arc<AtomicBool>,
}

impl RconClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            responses: Arc::new(Mutex::new(Vec::new())),
            reader_alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.reader_alive.load(Ordering::SeqCst)
    }

    pub fn connect(&mut self, host: &str, port: u16, password: &str) -> Result<String, String> {
        self.disconnect();

        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("地址无效: {}", e))?,
            Duration::from_secs(5),
        )
        .map_err(|e| format!("连接失败: {}", e))?;

        stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

        self.stream = Some(stream);

        // 读取欢迎信息
        let welcome = self.read_line_blocking().unwrap_or_default();

        if !welcome.contains("RocketRcon") {
            self.disconnect();
            return Err(format!("不是 Rocket RCON 服务器: {}", welcome));
        }

        // 发送认证
        self.write_line(&format!("login {}", password))?;

        // 等待足够时间让服务器响应认证结果
        std::thread::sleep(Duration::from_secs(2));
        if let Some(line) = self.try_read_line() {
            if line.contains("Invalid") || line.contains("not logged in") || line.contains("incorrect") {
                self.disconnect();
                return Err("密码错误".to_string());
            }
        }

        // 启动后台读取线程
        self.start_reader_thread();

        Ok(welcome)
    }

    pub fn send_command(&mut self, command: &str) -> Result<(), String> {
        self.write_line(command)
    }

    pub fn get_responses(&self) -> Vec<String> {
        let mut responses = self.responses.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *responses)
    }

    pub fn disconnect(&mut self) {
        self.stream = None;
        self.reader_alive.store(false, Ordering::SeqCst);
    }

    fn start_reader_thread(&self) {
        if let Some(ref stream) = self.stream {
            let stream_clone = stream.try_clone().ok();
            let responses = Arc::clone(&self.responses);
            let alive = Arc::clone(&self.reader_alive);

            if let Some(s) = stream_clone {
                // 读取线程用阻塞模式，不设超时
                s.set_read_timeout(None).ok();
                alive.store(true, Ordering::SeqCst);

                std::thread::spawn(move || {
                    let reader = BufReader::new(s);
                    for line in reader.lines() {
                        match line {
                            Ok(l) => {
                                let mut resp = responses.lock().unwrap_or_else(|e| e.into_inner());
                                if resp.len() > 1000 {
                                    resp.drain(0..500);
                                }
                                resp.push(l);
                            }
                            Err(_) => break,
                        }
                    }
                    alive.store(false, Ordering::SeqCst);
                });
            }
        }
    }

    fn write_line(&mut self, text: &str) -> Result<(), String> {
        if let Some(ref mut stream) = self.stream {
            let data = format!("{}\n", text);
            stream.write_all(data.as_bytes()).map_err(|e| format!("发送失败: {}", e))?;
            stream.flush().map_err(|e| format!("flush 失败: {}", e))?;
            Ok(())
        } else {
            Err("未连接".to_string())
        }
    }

    fn read_line_blocking(&mut self) -> Option<String> {
        if let Some(ref stream) = self.stream {
            let mut reader = BufReader::new(stream.try_clone().ok()?);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => None,
                Ok(_) => Some(line.trim_end().to_string()),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn try_read_line(&mut self) -> Option<String> {
        if let Some(ref stream) = self.stream {
            match stream.peek(&mut [0u8; 1]) {
                Ok(n) if n > 0 => self.read_line_blocking(),
                _ => None,
            }
        } else {
            None
        }
    }
}

// RconClient fields are all Send-safe (TcpStream + Arc<Mutex<Vec<String>>>)
// Removed manual unsafe impl Send — compiler auto-derives it correctly.
