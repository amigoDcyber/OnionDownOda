use crate::downloader::{self, DownloadProgress};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Idle,
    Downloading,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Input,
    Downloads,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    InProgress,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Download {
    pub filename: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub started_at: Instant,
}

impl Download {
    pub fn speed_bps(&self) -> f64 {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.downloaded_bytes as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn eta_seconds(&self) -> Option<u64> {
        let speed = self.speed_bps();
        if speed > 0.0 {
            if let Some(total) = self.total_bytes {
                let remaining = total.saturating_sub(self.downloaded_bytes);
                return Some((remaining as f64 / speed) as u64);
            }
        }
        None
    }
}

pub enum Action {
    None,
    StartDownload(String),
    Pause,
    Resume,
    Quit,
}

pub struct App {
    pub mode: AppMode,
    pub input: String,
    pub cursor_position: usize,
    pub downloads: Vec<Download>,
    pub log_messages: Vec<String>,
    pub tor_connected: bool,
    pub focus: Focus,
    pub should_quit: bool,
    pub proxy_addr: String,
    pub output_dir: PathBuf,
    pub progress_rx: Option<mpsc::UnboundedReceiver<DownloadProgress>>,
    pub download_scroll: u16,
    pub log_scroll: u16,
    pub paused: Arc<AtomicBool>,
}

impl App {
    pub fn new(proxy_addr: String, output_dir: PathBuf) -> Self {
        Self {
            mode: AppMode::Idle,
            input: String::new(),
            cursor_position: 0,
            downloads: Vec::new(),
            log_messages: vec!["🧅 Welcome to OnionDownOda".to_string()],
            tor_connected: false,
            focus: Focus::Input,
            should_quit: false,
            proxy_addr,
            output_dir,
            progress_rx: None,
            download_scroll: 0,
            log_scroll: 0,
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add_log(&mut self, msg: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let h = (now / 3600) % 24;
        let m = (now / 60) % 60;
        let s = now % 60;
        self.log_messages
            .push(format!("[{:02}:{:02}:{:02}] {}", h, m, s, msg));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::Quit;
        }

        match self.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::Downloads => self.handle_downloads_key(key),
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                let url = self.input.trim().to_string();
                if !url.is_empty() {
                    if !self.tor_connected {
                        self.add_log("⚠ Cannot download — Tor proxy not connected");
                        return Action::None;
                    }
                    self.input.clear();
                    self.cursor_position = 0;
                    return Action::StartDownload(url);
                }
                Action::None
            }
            KeyCode::Tab => {
                self.focus = Focus::Downloads;
                Action::None
            }
            KeyCode::Esc => Action::Quit,
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                }
                Action::None
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                }
                Action::None
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                Action::None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                Action::None
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                Action::None
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
                Action::None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_downloads_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Tab => {
                self.focus = Focus::Input;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char(' ') => {
                // Space toggles pause/resume
                if self.mode == AppMode::Downloading {
                    if self.paused.load(Ordering::Relaxed) {
                        return Action::Resume;
                    } else {
                        return Action::Pause;
                    }
                }
                Action::None
            }
            KeyCode::Up => {
                self.download_scroll = self.download_scroll.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                self.download_scroll = self.download_scroll.saturating_add(1);
                Action::None
            }
            KeyCode::PageUp => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
                Action::None
            }
            KeyCode::PageDown => {
                self.log_scroll = self.log_scroll.saturating_add(3);
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn start_download(&mut self, url: &str, rx: mpsc::UnboundedReceiver<DownloadProgress>) {
        let filename = downloader::extract_filename(url);
        self.downloads.push(Download {
            filename,
            total_bytes: None,
            downloaded_bytes: 0,
            status: DownloadStatus::InProgress,
            started_at: Instant::now(),
        });
        self.progress_rx = Some(rx);
        self.mode = AppMode::Downloading;
    }

    pub fn process_progress(&mut self) {
        // Drain channel into a vec first to avoid double mutable borrow
        let messages: Vec<DownloadProgress> = if let Some(rx) = &mut self.progress_rx {
            let mut msgs = Vec::new();
            while let Ok(progress) = rx.try_recv() {
                msgs.push(progress);
            }
            msgs
        } else {
            return;
        };

        for progress in messages {
            match progress {
                DownloadProgress::Started { filename, total_bytes } => {
                    let size_str = total_bytes
                        .map(format_bytes)
                        .unwrap_or_else(|| "unknown size".to_string());
                    self.add_log(&format!("📥 Starting: {} ({})", filename, size_str));
                    if let Some(dl) = self.downloads.last_mut() {
                        dl.total_bytes = total_bytes;
                        dl.filename = filename;
                    }
                }
                DownloadProgress::Progress { downloaded, total } => {
                    if let Some(dl) = self.downloads.last_mut() {
                        dl.downloaded_bytes = downloaded;
                        if dl.total_bytes.is_none() {
                            dl.total_bytes = total;
                        }
                    }
                }
                DownloadProgress::Completed { filepath, total_bytes } => {
                    if let Some(dl) = self.downloads.last_mut() {
                        dl.downloaded_bytes = total_bytes;
                        dl.status = DownloadStatus::Completed;
                    }
                    self.add_log(&format!(
                        "✅ Done: {} ({})",
                        filepath.display(),
                        format_bytes(total_bytes)
                    ));
                    self.mode = AppMode::Idle;
                }
                DownloadProgress::Failed { error } => {
                    if let Some(dl) = self.downloads.last_mut() {
                        dl.status = DownloadStatus::Failed(error.clone());
                    }
                    self.add_log(&format!("❌ Failed: {}", error));
                    self.mode = AppMode::Idle;
                }
            }
        }
    }

    pub fn pause_download(&mut self) {
        self.paused.store(true, Ordering::Relaxed);
        if let Some(dl) = self.downloads.last_mut() {
            if dl.status == DownloadStatus::InProgress {
                dl.status = DownloadStatus::Paused;
            }
        }
        self.add_log("⏸️ Download paused (Space to resume)");
    }

    pub fn resume_download(&mut self) {
        self.paused.store(false, Ordering::Relaxed);
        if let Some(dl) = self.downloads.last_mut() {
            if dl.status == DownloadStatus::Paused {
                dl.status = DownloadStatus::InProgress;
            }
        }
        self.add_log("▶️ Download resumed");
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
