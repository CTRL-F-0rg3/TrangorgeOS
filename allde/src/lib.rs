//! `allde` — a niri-like tiling desktop environment for TrangorgeOS userspace.
//!
//! Windows are tiled automatically into columns, can be moved between columns
//! and workspaces, are focused by keyboard or pointer, and every app runs as an
//! independent process. Multiple shell terminals can be opened at once.

pub mod font;
pub mod input;
pub mod process;
pub mod render;
pub mod shell;
pub mod wm;

use std::collections::HashMap;

use process::ProcessRegistry;
use render::{draw_cursor, draw_rect, draw_text, fill_rect, rgb, Color};
use shell::Shell;
use wm::Wm;

pub use input::{Cursor, InputEvent, Key, MouseButton};
pub use process::AppKind;
pub use wm::{Rect, WindowId};

#[cfg(test)]
mod tests;

const BG: Color = rgb(0x10, 0x12, 0x18);
const WIN_BG: Color = rgb(0x1E, 0x22, 0x2C);
const BORDER: Color = rgb(0x3A, 0x44, 0x5A);
const FOCUS_BORDER: Color = rgb(0x7C, 0xF0, 0x9C);
const TITLE_BG: Color = rgb(0x2A, 0x35, 0x4A);
const TITLE_TEXT: Color = rgb(0xD0, 0xD8, 0xE0);
const TEXT: Color = rgb(0xC8, 0xD0, 0xDC);
const PROMPT: Color = rgb(0x7C, 0xF0, 0x9C);

/// The whole desktop environment.
pub struct Allde {
    pub wm: Wm,
    pub procs: ProcessRegistry,
    pub shells: HashMap<WindowId, Shell>,
    pub cursor: Cursor,
    next_win_id: u32,
}

impl Allde {
    pub fn new(screen_w: u32, screen_h: u32) -> Self {
        Self {
            wm: Wm::new(screen_w, screen_h),
            procs: ProcessRegistry::new(),
            shells: HashMap::new(),
            cursor: Cursor::default(),
            next_win_id: 1,
        }
    }

    fn alloc_id(&mut self) -> WindowId {
        let id = self.next_win_id;
        self.next_win_id += 1;
        id
    }

    /// Spawn a shell terminal as a new process + window.
    pub fn spawn_shell(&mut self, title: &str) -> WindowId {
        let id = self.alloc_id();
        self.wm.add(id);
        self.procs.spawn(AppKind::Shell, id, title.to_string());
        self.shells.insert(id, Shell::new("user@allde:~$ "));
        id
    }

    /// Spawn a non-interactive app (status bar / info panel).
    pub fn spawn_app(&mut self, kind: AppKind, title: &str) -> WindowId {
        let id = self.alloc_id();
        self.wm.add(id);
        self.procs.spawn(kind, id, title.to_string());
        id
    }

    pub fn close_window(&mut self, id: WindowId) {
        self.wm.remove(id);
        self.shells.remove(&id);
        self.procs.remove_by_window(id);
    }

    pub fn handle_input(&mut self, ev: InputEvent) {
        match ev {
            InputEvent::Key(k) => self.handle_key(k),
            InputEvent::MouseMove(x, y) => {
                self.cursor.x = x;
                self.cursor.y = y;
                if let Some(id) = self.wm.window_at(x, y) {
                    self.wm.focus(id);
                }
            }
            InputEvent::MouseButton(MouseButton::Left, true) => {
                if let Some(id) = self.wm.window_at(self.cursor.x, self.cursor.y) {
                    self.wm.focus(id);
                }
            }
            _ => {}
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        match key {
            Key::Char(c) => self.type_char(c),
            Key::Enter => self.type_char('\n'),
            Key::Backspace => self.type_char('\x08'),
            Key::Left => {
                if let Some(id) = self.wm.focused {
                    self.wm.move_left(id);
                }
            }
            Key::Right => {
                if let Some(id) = self.wm.focused {
                    self.wm.move_right(id);
                }
            }
            Key::Up => self.wm.focus_prev(),
            Key::Down => self.wm.focus_next(),
            _ => {}
        }
    }

    pub fn type_char(&mut self, c: char) {
        let id = match self.wm.focused {
            Some(id) => id,
            None => return,
        };

        if c == '\n' {
            let input = self
                .shells
                .get(&id)
                .map(|s| s.input.clone())
                .unwrap_or_default();
            if let Some(out) = self.run_wm_command(id, &input) {
                if let Some(s) = self.shells.get_mut(&id) {
                    s.lines.push(format!("{}{}", s.prompt, input));
                    s.lines.extend(out);
                    s.input.clear();
                }
                return;
            }
        }

        let closed = match self.shells.get_mut(&id) {
            Some(shell) => {
                shell.feed_char(c);
                shell.closed
            }
            None => false,
        };
        if closed {
            self.close_window(id);
        }
    }

    /// Desktop-level commands typed into a shell (spawn / kill / list).
    fn run_wm_command(&mut self, id: WindowId, line: &str) -> Option<Vec<String>> {
        let t = line.trim();
        if t == "new" {
            self.spawn_shell("terminal");
            return Some(vec!["spawned a new terminal".to_string()]);
        }
        if t == "list" {
            let out = self
                .procs
                .list()
                .iter()
                .map(|(pid, title)| format!("pid {:<3} {}", pid, title))
                .collect();
            return Some(out);
        }
        if let Some(rest) = t.strip_prefix("kill ") {
            if let Ok(pid) = rest.trim().parse::<u32>() {
                return match self.procs.kill(pid) {
                    Some(w) => {
                        self.close_window(w);
                        Some(vec![format!("killed pid {}", pid)])
                    }
                    None => Some(vec![format!("no such pid {}", pid)]),
                };
            }
        }
        let _ = id;
        None
    }

    /// Render a frame of the desktop into the framebuffer.
    pub fn render(&self, fb: &mut [u32]) {
        let w = self.wm.screen_w;
        let h = self.wm.screen_h;
        fill_rect(fb, w, h, 0, 0, w, h, BG);

        for (id, rect) in self.wm.layout() {
            self.render_window(fb, id, &rect);
        }

        if self.cursor.visible {
            draw_cursor(fb, w, h, self.cursor.x, self.cursor.y);
        }
    }

    fn render_window(&self, fb: &mut [u32], id: WindowId, r: &Rect) {
        let w = self.wm.screen_w;
        let h = self.wm.screen_h;
        let focused = self.wm.focused == Some(id);

        fill_rect(fb, w, h, r.x, r.y, r.w, r.h, WIN_BG);
        draw_rect(
            fb, w, h, r.x, r.y, r.w, r.h,
            if focused { FOCUS_BORDER } else { BORDER },
        );
        fill_rect(fb, w, h, r.x + 1, r.y + 1, r.w - 2, 20, TITLE_BG);

        let title = self
            .procs
            .by_window(id)
            .map(|a| a.title.clone())
            .unwrap_or_default();
        draw_text(fb, w, h, r.x + 6, r.y + 4, &title, TITLE_TEXT);

        match self.procs.by_window(id).map(|a| a.kind) {
            Some(AppKind::Shell) => {
                if let Some(shell) = self.shells.get(&id) {
                    let max_lines = ((r.h.saturating_sub(24)) / 10).max(1) as usize;
                    let lines = shell.visible_lines(max_lines);
                    let mut ty = r.y + 26;
                    for line in &lines {
                        let color = if line.starts_with(&shell.prompt) {
                            PROMPT
                        } else {
                            TEXT
                        };
                        draw_text(fb, w, h, r.x + 6, ty, line, color);
                        ty += 10;
                    }
                }
            }
            Some(AppKind::StatusBar) => {
                draw_text(
                    fb, w, h, r.x + 6, r.y + 28,
                    &format!("procs: {}", self.procs.count()),
                    TEXT,
                );
            }
            Some(AppKind::Info) => {
                draw_text(fb, w, h, r.x + 6, r.y + 28, "all-de desktop", TEXT);
                draw_text(fb, w, h, r.x + 6, r.y + 38, "niri-like tiling", TEXT);
            }
            None => {}
        }
    }
}
