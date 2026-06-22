use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// RCON 响应缓冲区最大条目数，超过后裁剪旧数据
const MAX_RESPONSE_ENTRIES: usize = 500;
/// 每次裁剪的条目数
const RESPONSE_TRIM_BATCH: usize = 250;
/// 单条响应最大字符数，防止超长响应占用过多内存
const MAX_RESPONSE_LENGTH: usize = 1000;
/// 心跳间隔（秒），读超时后发送一条无副作用命令保持连接活跃
/// 服务器 read 循环有 100ms Thread.Sleep，心跳响应会在 ~200ms 内返回
const HEARTBEAT_INTERVAL_SECS: u64 = 60;
/// 心跳命令：使用对在线玩家无可见副作用的只读查询命令保活
/// （原先发送空 `say` 会向玩家推送空白聊天）
const HEARTBEAT_COMMAND: &[u8] = b"players\n";

/// Rocket RCON TCP 客户端，支持连接、认证、命令发送、后台响应读取和心跳保活
///
/// 读写分离：写端（`stream`）由主结构持有并用 Mutex 保护，隔离命令写入与心跳写入；
/// 读端从 connect 阶段就独占一个 `BufReader`，认证完成后整体移交后台线程，
/// 因此 `BufReader` 预读的数据不会在认证/读取交接时丢失。
pub struct RconClient {
    /// 共享的 TCP 写入端
    stream: Option<Arc<Mutex<TcpStream>>>,
    responses: Arc<Mutex<Vec<String>>>,
    reader_alive: Arc<AtomicBool>,
    connected_save_id: Option<String>,
}

impl RconClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            responses: Arc::new(Mutex::new(Vec::new())),
            reader_alive: Arc::new(AtomicBool::new(false)),
            connected_save_id: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.reader_alive.load(Ordering::SeqCst)
    }

    /// 连接到 Rocket RCON 服务器并认证，返回欢迎信息
    pub fn connect(&mut self, host: &str, port: u16, password: &str) -> Result<String, String> {
        self.disconnect();

        let addr = format!("{}:{}", host, port);
        let raw_stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("地址无效: {}", e))?,
            Duration::from_secs(2),
        )
        .map_err(|e| format!("连接失败: {}", e))?;

        // 读端独立 clone：认证阶段就用它，之后整体移交后台线程，缓冲不丢
        let read_half = raw_stream
            .try_clone()
            .map_err(|e| format!("克隆连接失败: {}", e))?;
        // 认证阶段读超时设短一些，便于在无响应时尽快判定失败
        read_half
            .set_read_timeout(Some(Duration::from_secs(2)))
            .ok();
        raw_stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .ok();

        let write_stream = Arc::new(Mutex::new(raw_stream));
        self.stream = Some(Arc::clone(&write_stream));

        let mut reader = BufReader::new(read_half);

        // 读取欢迎信息
        let mut welcome = String::new();
        if reader.read_line(&mut welcome).is_err() {
            self.disconnect();
            return Err("读取欢迎信息失败".to_string());
        }
        let welcome = welcome.trim_end().to_string();

        if !welcome.contains("RocketRcon") {
            self.disconnect();
            return Err(format!("不是 Rocket RCON 服务器: {}", welcome));
        }

        // 发送认证
        self.write_line_to(&write_stream, &format!("login {}", password))?;

        // 显式等待登录结果：命中失败标志立即返回错误，命中成功标志确认通过；
        // 其他行（如广播）继续读，不再用“非失败即成功”的判定。
        let mut authed = false;
        for _ in 0..10 {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let l = line.trim();
                    if l.is_empty() {
                        continue;
                    }
                    if l.contains("Invalid")
                        || l.contains("not logged in")
                        || l.contains("incorrect")
                        || l.contains("Wrong password")
                    {
                        self.disconnect();
                        return Err("密码错误".to_string());
                    }
                    if l.contains("logged in")
                        || l.contains("Authenticated")
                        || l.contains("Login successful")
                        || l.contains("Welcome")
                    {
                        authed = true;
                        break;
                    }
                    // 其他响应：继续等待明确的成功/失败标志
                }
                Err(ref e)
                    if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock =>
                {
                    break;
                }
                Err(e) => {
                    self.disconnect();
                    return Err(format!("认证读取失败: {}", e));
                }
            }
        }

        if !authed {
            self.disconnect();
            return Err("认证未确认成功（未收到登录成功响应，请检查密码）".to_string());
        }

        // 把 reader（含其内部缓冲）整体移交后台线程
        self.start_reader_thread(reader, Arc::clone(&write_stream));

        Ok(welcome)
    }

    pub fn send_command(&mut self, command: &str) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            self.write_line_to(stream, command)
        } else {
            Err("未连接".to_string())
        }
    }

    pub fn get_responses(&self) -> Vec<String> {
        let mut responses = self.responses.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *responses)
    }

    pub fn set_connected_save_id(&mut self, save_id: String) {
        self.connected_save_id = Some(save_id);
    }

    pub fn connected_save_id(&self) -> Option<String> {
        if self.is_connected() {
            self.connected_save_id.clone()
        } else {
            None
        }
    }

    pub fn disconnect(&mut self) {
        // 关闭 TCP 连接，使读线程的 read 操作返回错误并退出
        if let Some(ref stream) = self.stream {
            if let Ok(s) = stream.lock() {
                let _ = s.shutdown(std::net::Shutdown::Both);
            }
        }
        self.stream = None;
        self.connected_save_id = None;
        self.reader_alive.store(false, Ordering::SeqCst);
    }

    /// 后台读取线程：独占 `reader`（含其内部缓冲），用 `read_line` 循环读取。
    /// 读超时（达到心跳间隔）时通过写端发送一条无副作用命令保活。
    fn start_reader_thread(
        &self,
        mut reader: BufReader<TcpStream>,
        write_stream: Arc<Mutex<TcpStream>>,
    ) {
        let responses = Arc::clone(&self.responses);
        let alive = Arc::clone(&self.reader_alive);

        // 读端用心跳间隔做超时
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)))
            .ok();
        alive.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // EOF，连接已关闭
                    Ok(_) => {
                        let l = line.trim_end().to_string();
                        if !l.is_empty() {
                            let mut resp = responses.lock().unwrap_or_else(|e| e.into_inner());
                            if resp.len() > MAX_RESPONSE_ENTRIES {
                                resp.drain(0..RESPONSE_TRIM_BATCH);
                            }
                            // 限制单条响应长度，防止超长响应占用过多内存
                            let truncated = if l.len() > MAX_RESPONSE_LENGTH {
                                l.chars().take(MAX_RESPONSE_LENGTH).collect()
                            } else {
                                l
                            };
                            resp.push(truncated);
                        }
                    }
                    Err(ref e)
                        if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock =>
                    {
                        // 读取超时 = 心跳间隔到了，发送无副作用命令保持连接
                        if let Ok(mut stream) = write_stream.lock() {
                            let _ = stream.write_all(HEARTBEAT_COMMAND);
                            let _ = stream.flush();
                        }
                        continue;
                    }
                    Err(_) => break,
                }
            }
            alive.store(false, Ordering::SeqCst);
        });
    }

    fn write_line_to(&self, stream: &Arc<Mutex<TcpStream>>, text: &str) -> Result<(), String> {
        let mut s = stream.lock().map_err(|_| "连接锁中毒".to_string())?;
        let data = format!("{}\n", text);
        s.write_all(data.as_bytes())
            .map_err(|e| format!("发送失败: {}", e))?;
        s.flush().map_err(|e| format!("flush 失败: {}", e))?;
        Ok(())
    }
}
