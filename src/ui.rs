use crate::app::{App, DownloadStatus, Focus, format_bytes};
use crate::banner::BANNER;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

// ── Color Palette (as specified) ──────────────────────────────
// Background: Black, Logo: Magenta/Pink, Borders: Magenta
// Labels: Magenta, Status: Green, Input/Social labels: Cyan
// URLs/Text: White/Gray, Warning: Yellow
const LOGO_MAGENTA: Color = Color::Rgb(255, 0, 255);
const LOGO_PINK: Color = Color::Rgb(255, 110, 199);
const MAGENTA: Color = Color::Rgb(191, 64, 255);
const CYAN: Color = Color::Rgb(0, 255, 255);
const GREEN: Color = Color::Rgb(57, 255, 20);
const YELLOW: Color = Color::Rgb(255, 200, 0);
const WHITE: Color = Color::White;
const GRAY: Color = Color::Rgb(180, 180, 180);
const DIM_GRAY: Color = Color::Rgb(80, 80, 80);
const DARK_BG: Color = Color::Rgb(0, 0, 0);
const SURFACE: Color = Color::Rgb(10, 10, 10);

// Social links data
const SOCIALS: &[(&str, &str, &str)] = &[
    ("[1]", "Instagram", "https://www.instagram.com/amigo.d.cyber"),
    ("[2]", "Facebook", "https://www.facebook.com/amigo.d.cyber"),
    ("[3]", "YouTube", "https://www.youtube.com/@CyberMafiaX"),
    ("[4]", "TikTok", "https://www.tiktok.com/@amigo.d.cyber"),
    ("[5]", "X/Twitter", "https://x.com/MafiaCyberX"),
    ("[6]", "Snapchat", "https://www.snapchat.com/add/amigo-cyber"),
    ("[7]", "Pinterest", "https://www.pinterest.com/amigodcyber/"),
    ("[8]", "GitHub", "https://github.com/amigoDcyber"),
    ("[0]", "Linktree", "https://linktr.ee/Amigo.D.Cyber"),
];

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ── Main draw entry point ──────────────────────────────────────
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let width = size.width;

    // Overall dark background
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0))),
        size,
    );

    // Determine layout mode based on terminal width
    let layout_mode = if width >= 120 {
        LayoutMode::Wide
    } else if width >= 80 {
        LayoutMode::Medium
    } else {
        LayoutMode::Narrow
    };

    // Percentage-based responsive layout (Developer & Socials panel removed)
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(28), // Header (banner + socials + credit) - increased
            Constraint::Percentage(7),  // Tor status
            Constraint::Percentage(10), // Input area
            Constraint::Percentage(32), // Downloads
            Constraint::Percentage(12), // Log (smaller)
            // Socials panel removed - links now in header
        ])
        .split(size);

    // Draw sections
    match layout_mode {
        LayoutMode::Wide => draw_header_wide(frame, main_layout[0]),
        LayoutMode::Medium => draw_header_medium(frame, main_layout[0]),
        LayoutMode::Narrow => draw_header_narrow(frame, main_layout[0]),
    }

    draw_tor_status(frame, app, main_layout[1]);
    draw_input(frame, app, main_layout[2]);
    draw_downloads(frame, app, main_layout[3]);
    draw_log(frame, app, main_layout[4]);
    draw_disclaimer_and_help(frame, app, size);
}

#[derive(Clone, Copy)]
enum LayoutMode {
    Wide,   // >= 120 cols
    Medium, // 80-119 cols
    Narrow, // < 80 cols
}

// ── Wide Header: 3-column (socials | logo | socials) ─────────────
fn draw_header_wide(frame: &mut Frame, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Left socials (1-4)
    draw_social_column(frame, chunks[0], 0..4);
    // Center logo
    draw_logo_centered(frame, chunks[1]);
    // Right socials (5-8 + Linktree)
    draw_social_column(frame, chunks[2], 4..9);
}

// ── Medium Header: Logo centered, socials below ─────────────────
fn draw_header_medium(frame: &mut Frame, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_logo_centered(frame, chunks[0]);
    draw_socials_compact(frame, chunks[1]);
}

// ── Narrow Header: Plain text logo + warning ───────────────────
fn draw_header_narrow(frame: &mut Frame, area: ratatui::layout::Rect) {
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            "[ OnionDownOda ]",
            Style::default()
                .fg(LOGO_MAGENTA)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(Span::styled(
            "⚠️  Please resize terminal (min 80 cols) for full UI",
            Style::default().fg(YELLOW),
        )),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(text, area);
}

// ── Logo with DownOda subtitle ─────────────────────────────────
fn draw_logo_centered(frame: &mut Frame, area: ratatui::layout::Rect) {
    let mut lines = Vec::new();
    let max_i = (BANNER.len() as f32 - 1.0).max(1.0);

    // Full gradient banner - always show the ASCII art
    for (i, line_str) in BANNER.iter().enumerate() {
        let t = i as f32 / max_i;
        let r = lerp(255.0, 191.0, t) as u8;
        let g = lerp(0.0, 64.0, t) as u8;
        let b = lerp(255.0, 255.0, t) as u8;
        let color = Color::Rgb(r, g, b);

        lines.push(Line::from(Span::styled(
            *line_str,
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Add credit line below the banner
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "made by Amigo.D.Cyber",
        Style::default().fg(GRAY).add_modifier(Modifier::ITALIC),
    )));

    let banner = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(DARK_BG));

    frame.render_widget(banner, area);
}

// ── Social column for wide layout ──────────────────────────────
fn draw_social_column(frame: &mut Frame, area: ratatui::layout::Rect, range: std::ops::Range<usize>) {
    let mut lines = Vec::new();
    for i in range {
        if let Some((key, platform, url)) = SOCIALS.get(i) {
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", key), Style::default().fg(MAGENTA)),
                Span::styled(platform.to_string(), Style::default().fg(CYAN)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ".to_string(), Style::default()),
                Span::styled(url.to_string(), Style::default().fg(GRAY)),
            ]));
            lines.push(Line::from(""));
        }
    }
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(Color::Rgb(0, 0, 0))),
        area,
    );
}

// ── Compact socials for medium layout ──────────────────────────
fn draw_socials_compact(frame: &mut Frame, area: ratatui::layout::Rect) {
    let mut lines = Vec::new();
    let mut current_line = vec![Span::styled("  ", Style::default())];

    for (i, (key, platform, _url)) in SOCIALS.iter().enumerate() {
        current_line.push(Span::styled(
            format!("{} ", key),
            Style::default().fg(MAGENTA),
        ));
        current_line.push(Span::styled(
            format!("{}  ", platform),
            Style::default().fg(CYAN),
        ));

        // New line every 3 items
        if (i + 1) % 3 == 0 || i == SOCIALS.len() - 1 {
            lines.push(Line::from(current_line.clone()));
            current_line = vec![Span::styled("  ", Style::default())];
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(DARK_BG)),
        area,
    );
}

// ── Tor Status Bar ─────────────────────────────────────────────
fn draw_tor_status(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (icon, text, color) = if app.tor_connected {
        ("●", format!(" Connected ({})", app.proxy_addr), GREEN)
    } else {
        ("●", " Disconnected — start tor service".to_string(), Color::Red)
    };

    let line = Line::from(vec![
        Span::styled(
            "  🧅 Tor Status: ",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ),
        Span::styled(icon, Style::default().fg(color)),
        Span::styled(text, Style::default().fg(color)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAGENTA))
        .style(Style::default().bg(SURFACE));

    frame.render_widget(Paragraph::new(line).block(block), area);
}

// ── URL Input Field ────────────────────────────────────────────
fn draw_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let focused = app.focus == Focus::Input;
    let border_color = if focused { CYAN } else { DIM_GRAY };
    let title_style = if focused {
        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MAGENTA)
    };

    let block = Block::default()
        .title(Span::styled(" 📎 Paste .onion URL ", title_style))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(SURFACE));

    let display_text = if app.input.is_empty() && !focused {
        Line::from(Span::styled(
            "  Enter a .onion URL and press Enter to download...",
            Style::default().fg(DIM_GRAY).add_modifier(Modifier::ITALIC),
        ))
    } else {
        Line::from(Span::styled(
            format!(" {}", &app.input),
            Style::default().fg(WHITE),
        ))
    };

    frame.render_widget(
        Paragraph::new(display_text).block(block).wrap(Wrap { trim: false }),
        area,
    );

    // Cursor position when focused
    if focused {
        let x = area.x + app.cursor_position as u16 + 2;
        let y = area.y + 1;
        frame.set_cursor_position((x, y));
    }
}

// ── Downloads Panel ────────────────────────────────────────────
fn draw_downloads(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .title(Span::styled(
            " 📥 Downloads ",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAGENTA))
        .style(Style::default().bg(SURFACE));

    if app.downloads.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  No downloads yet — paste a URL above and hit Enter",
            Style::default().fg(DIM_GRAY).add_modifier(Modifier::ITALIC),
        )))
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    for dl in &app.downloads {
        let icon = match &dl.status {
            DownloadStatus::InProgress => "⏳",
            DownloadStatus::Paused => "⏸️",
            DownloadStatus::Completed => "✅",
            DownloadStatus::Failed(_) => "❌",
        };

        // Filename row
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", icon), Style::default().fg(WHITE)),
            Span::styled(
                &dl.filename,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Progress / status row
        match &dl.status {
            DownloadStatus::InProgress | DownloadStatus::Paused => {
                let is_paused = dl.status == DownloadStatus::Paused;
                let speed = if is_paused { 0.0 } else { dl.speed_bps() };
                let speed_str = format!("{}/s", format_bytes(speed as u64));

                if let Some(total) = dl.total_bytes {
                    let ratio = dl.downloaded_bytes as f64 / total as f64;
                    let bar_width = (inner.width as usize).saturating_sub(30).max(10);
                    let filled = (ratio * bar_width as f64) as usize;
                    let empty = bar_width.saturating_sub(filled);
                    let eta_str = if is_paused {
                        "PAUSED".to_string()
                    } else {
                        dl.eta_seconds()
                            .map(|s| format!("ETA {}s", s))
                            .unwrap_or_default()
                    };
                    let pct = (ratio * 100.0) as u32;
                    let bar_color = if is_paused { Color::Red } else { GREEN };

                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
                        Span::styled("░".repeat(empty), Style::default().fg(DIM_GRAY)),
                        Span::styled(
                            format!(" {:>3}%  {}  {}", pct, speed_str, eta_str),
                            Style::default().fg(if is_paused { Color::Red } else { CYAN }),
                        ),
                    ]));
                } else {
                    if is_paused {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  ⏸️ Paused ".to_string(),
                                Style::default().fg(Color::Red),
                            ),
                            Span::styled(
                                format!("{}  {}", format_bytes(dl.downloaded_bytes), speed_str),
                                Style::default().fg(CYAN),
                            ),
                        ]));
                    } else {
                        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                        let spin_idx = (dl.started_at.elapsed().as_millis() / 100) as usize % spinner.len();
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {} Downloading ", spinner[spin_idx]),
                                Style::default().fg(GREEN),
                            ),
                            Span::styled(
                                format!("{}  {}", format_bytes(dl.downloaded_bytes), speed_str),
                                Style::default().fg(CYAN),
                            ),
                        ]));
                    }
                }
            }
            DownloadStatus::Completed => {
                lines.push(Line::from(Span::styled(
                    format!("  ✓ Done ({})", format_bytes(dl.downloaded_bytes)),
                    Style::default().fg(GREEN),
                )));
            }
            DownloadStatus::Failed(err) => {
                lines.push(Line::from(Span::styled(
                    format!("  ✗ {}", err),
                    Style::default().fg(Color::Red),
                )));
            }
        }
        lines.push(Line::from(""));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.download_scroll, 0)),
        inner,
    );
}

// ── Log Panel (shrunk to 3-4 lines with scrollback) ────────────
fn draw_log(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .title(Span::styled(
            " 📋 Log (scroll ↑↓ PgUp/PgDn) ",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 40)))
        .style(Style::default().bg(Color::Rgb(10, 10, 10)));

    let max_visible = 4;
    let total_logs = app.log_messages.len();
    let scroll = app.log_scroll as usize;
    
    // Calculate visible range
    let start = scroll.min(total_logs.saturating_sub(max_visible));
    let end = (start + max_visible).min(total_logs);
    
    let log_lines: Vec<Line> = app
        .log_messages
        .iter()
        .skip(start)
        .take(end - start)
        .map(|msg| {
            let color = if msg.contains('✅') || msg.contains("Connected") {
                GREEN
            } else if msg.contains('❌') || msg.contains('⚠') {
                Color::Red
            } else if msg.contains("📥") || msg.contains("🔗") {
                CYAN
            } else {
                GRAY
            };
            Line::from(Span::styled(
                format!("  {}", msg),
                Style::default().fg(color),
            ))
        })
        .collect();

    frame.render_widget(Paragraph::new(log_lines).block(block), area);
}

// ── Socials Panel (full URLs visible) ────────────────────────────
#[allow(dead_code)]
fn draw_socials_panel(frame: &mut Frame, area: ratatui::layout::Rect, mode: LayoutMode) {
    let block = Block::default()
        .title(Span::styled(
            " 👤 Developer & Socials — Press keys to open in browser ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAGENTA))
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // GitHub credit line
    lines.push(Line::from(vec![
        Span::styled("  GitHub: ", Style::default().fg(CYAN)),
        Span::styled(
            "@amigoDcyber",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — High-speed Tor downloader with Rust", Style::default().fg(GRAY)),
    ]));
    lines.push(Line::from(""));

    match mode {
        LayoutMode::Wide => {
            // Two columns of social links: 4 left, 5 right
            let left = &SOCIALS[0..4];   // [1-4] Instagram, Facebook, YouTube, TikTok
            let right = &SOCIALS[4..9];  // [5-8] + [0] X, Snapchat, Pinterest, GitHub, Linktree
            
            // 5 rows to accommodate all 9 items
            for i in 0..5 {
                let left_str = if let Some((k, p, u)) = left.get(i) {
                    format!("{} {} → {}", k, p, u)
                } else {
                    String::new()  // Empty for row 5 since left only has 4
                };
                let right_str = if let Some((k, p, u)) = right.get(i) {
                    format!("{} {} → {}", k, p, u)
                } else {
                    String::new()
                };
                
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", left_str), Style::default().fg(WHITE)),
                    Span::styled("    ", Style::default()),
                    Span::styled(right_str, Style::default().fg(WHITE)),
                ]));
            }
        }
        _ => {
            // Single column for medium/narrow
            for (key, platform, url) in SOCIALS.iter() {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", key), Style::default().fg(MAGENTA)),
                    Span::styled(format!("{} ", platform), Style::default().fg(CYAN)),
                    Span::styled(format!("→ {}", url), Style::default().fg(WHITE)),
                ]));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(SURFACE)),
        inner,
    );
}

// ── Disclaimer Banner + Help Bar ────────────────────────────────
fn draw_disclaimer_and_help(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Use the bottom area of the screen
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Min(1),  // Push to bottom
            Constraint::Length(2), // Disclaimer + Help
        ])
        .split(area);

    let footer_area = bottom[1];

    // Disclaimer line (yellow warning)
    let disclaimer = Paragraph::new(Line::from(vec![
        Span::styled(" ⚠️  ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(
            "The developer is NOT responsible for any files downloaded without proper authorization.",
            Style::default().fg(YELLOW),
        ),
    ]));

    frame.render_widget(disclaimer, footer_area);

    // Help bar at very bottom
    if footer_area.height >= 2 {
        let help_area = ratatui::layout::Rect {
            x: footer_area.x,
            y: footer_area.y + 1,
            width: footer_area.width,
            height: 1,
        };

        let focus_label = match app.focus {
            Focus::Input => "INPUT",
            Focus::Downloads => "DOWNLOADS",
        };

        let help = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Download  ", Style::default().fg(WHITE)),
            Span::styled("[Space]", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Pause  ", Style::default().fg(WHITE)),
            Span::styled("[Tab]", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Focus  ", Style::default().fg(WHITE)),
            Span::styled("[0-8]", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled(" Socials  ", Style::default().fg(WHITE)),
            Span::styled("[↑↓]", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Scroll  ", Style::default().fg(WHITE)),
            Span::styled("[Esc]", Style::default().fg(LOGO_PINK).add_modifier(Modifier::BOLD)),
            Span::styled(" Quit  ", Style::default().fg(WHITE)),
            Span::styled(
                format!("▸ {}", focus_label),
                Style::default()
                    .fg(MAGENTA)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        frame.render_widget(
            Paragraph::new(help).style(Style::default().bg(DARK_BG)),
            help_area,
        );
    }
}
