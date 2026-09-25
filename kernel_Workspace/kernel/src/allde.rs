//! Interactive `all-de` desktop: a niri-like tiling environment that runs in
//! kernel space and takes over input from the system terminal.
//!
//! Keys: arrows move the cursor, F4 clicks (focus), Tab focuses next window,
//! F1 minimizes, F2 closes, F3 opens a window, ESC returns to the terminal.
//! Typing goes into the focused window's shell (not the kernel terminal).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::gfx::font::FONT8X8;
use crate::gfx::framebuffer::{rgb, Framebuffer, PixelFormat};

extern "C" {
    fn mouse_init();
    fn mouse_poll(dx: *mut i32, dy: *mut i32, dz: *mut i32, buttons: *mut i32) -> i32;
}

const PROMPT: &str = "user@allde:~$ ";
const BAR_H: u32 = 22;
const TITLE_H: u32 = 20;
const RADIUS: i32 = 4;
const CURSOR_W: u32 = 16;
const CURSOR_H: u32 = 16;

const KEY_ENTER: u32 = 0x100;
const KEY_BACKSPACE: u32 = 0x101;
const KEY_ESC: u32 = 0x102;
const KEY_RIGHT: u32 = 0x103;
const KEY_LEFT: u32 = 0x104;
const KEY_DOWN: u32 = 0x105;
const KEY_UP: u32 = 0x106;
const KEY_TAB: u32 = 0x10A;
const KEY_F1: u32 = 0x112;
const KEY_F2: u32 = 0x113;
const KEY_F3: u32 = 0x114;
const KEY_F4: u32 = 0x115;

struct Window {
    title: String,
    lines: Vec<String>,
    input: String,
    minimized: bool,
}

impl Window {
    fn new(title: &str) -> Self {
        let mut w = Self {
            title: title.to_string(),
            lines: Vec::new(),
            input: String::new(),
            minimized: false,
        };
        w.lines.push("TrangorgeOS all-de shell".into());
        w.lines.push("type 'help' for commands".into());
        w
    }

    fn visible_lines(&self, max: usize) -> Vec<String> {
        let mut v = Vec::new();
        let start = self.lines.len().saturating_sub(max);
        for l in &self.lines[start..] {
            v.push(l.clone());
        }
        v.push(format!("{}{}", PROMPT, self.input));
        v
    }
}

struct Desktop {
    windows: Vec<Window>,
    focused: usize,
    cursor_x: u32,
    cursor_y: u32,
}

impl Desktop {
    fn new() -> Self {
        let mut d = Self {
            windows: Vec::new(),
            focused: 0,
            cursor_x: 120,
            cursor_y: 120,
        };
        d.add_window("terminal 1");
        d
    }

    fn add_window(&mut self, title: &str) {
        self.windows.push(Window::new(title));
        self.focused = self.windows.len() - 1;
    }

    fn close_focused(&mut self) {
        if self.focused < self.windows.len() {
            self.windows.remove(self.focused);
        }
        if self.windows.is_empty() {
            self.windows.push(Window::new("terminal"));
        }
        if self.focused >= self.windows.len() {
            self.focused = self.windows.len() - 1;
        }
    }

    fn minimize_focused(&mut self) {
        if self.focused < self.windows.len() {
            let w = &mut self.windows[self.focused];
            w.minimized = !w.minimized;
        }
    }

    fn focus_next(&mut self) {
        let n = self.windows.len();
        if n == 0 {
            return;
        }
        for step in 1..=n {
            let idx = (self.focused + step) % n;
            if !self.windows[idx].minimized {
                self.focused = idx;
                return;
            }
        }
    }

    fn click(&mut self, sw: u32, sh: u32) {
        let visible: Vec<usize> = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, w)| !w.minimized)
            .map(|(i, _)| i)
            .collect();
        let n = visible.len().max(1) as u32;
        let col_w = sw / n;
        for (i, &idx) in visible.iter().enumerate() {
            let x = i as u32 * col_w;
            if self.cursor_x >= x
                && self.cursor_x < x + col_w
                && self.cursor_y >= BAR_H
                && self.cursor_y < sh
            {
                self.focused = idx;
                return;
            }
        }
    }

    fn type_char(&mut self, c: char) {
        if self.focused >= self.windows.len() {
            return;
        }
        match c {
            '\n' => self.submit(),
            '\x08' => {
                self.windows[self.focused].input.pop();
            }
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => {
                self.windows[self.focused].input.push(c);
            }
            _ => {}
        }
    }

    fn submit(&mut self) {
        if self.focused >= self.windows.len() {
            return;
        }
        let line = core::mem::take(&mut self.windows[self.focused].input);
        let trimmed = line.trim();
        let (cmd, rest) = split_once_space(trimmed);

        match cmd {
            "new" => {
                self.windows[self.focused].lines.push(format!("{}{}", PROMPT, line));
                self.windows[self.focused].lines.push("spawned a new terminal".into());
                self.add_window("terminal");
                return;
            }
            "exit" => {
                self.close_focused();
                return;
            }
            "list" => {
                let mut out = vec![format!("{}{}", PROMPT, line)];
                for (i, win) in self.windows.iter().enumerate() {
                    let mark = if i == self.focused { "*" } else { " " };
                    out.push(format!("{} [{}] {}", mark, i, win.title));
                }
                self.windows[self.focused].lines.extend(out);
                return;
            }
            _ => {}
        }

        let out: Vec<String> = match cmd {
            "help" => vec![
                "commands: help echo clear uname new list exit".into(),
                "keys: arrows=cursor F4=click Tab=focus".into(),
                "      F1=minimize F2=close F3=new ESC=quit".into(),
            ],
            "echo" => vec![rest.to_string()],
            "clear" => {
                self.windows[self.focused].lines.clear();
                vec![]
            }
            "uname" => vec!["TrangorgeOS all-de desktop".into()],
            _ => vec![format!("{}: command not found (try 'help')", cmd)],
        };
        self.windows[self.focused].lines.push(format!("{}{}", PROMPT, line));
        self.windows[self.focused].lines.extend(out);
    }

    fn handle_keycode(&mut self, k: u32, sw: u32, sh: u32) -> bool {
        match k {
            KEY_ESC => return true,
            KEY_ENTER => self.type_char('\n'),
            KEY_BACKSPACE => self.type_char('\x08'),
            KEY_TAB => self.focus_next(),
            KEY_F1 => self.minimize_focused(),
            KEY_F2 => self.close_focused(),
            KEY_F3 => self.add_window("terminal"),
            KEY_F4 => self.click(sw, sh),
            KEY_RIGHT => self.cursor_x = (self.cursor_x + 8).min(sw.saturating_sub(8)),
            KEY_LEFT => self.cursor_x = self.cursor_x.saturating_sub(8),
            KEY_DOWN => self.cursor_y = (self.cursor_y + 8).min(sh.saturating_sub(8)),
            KEY_UP => self.cursor_y = self.cursor_y.saturating_sub(8),
            c if c >= 0x20 && c < 0x7F => self.type_char(c as u8 as char),
            _ => {}
        }
        false
    }

    fn visible_indices(&self) -> Vec<usize> {
        self.windows
            .iter()
            .enumerate()
            .filter(|(_, w)| !w.minimized)
            .map(|(i, _)| i)
            .collect()
    }

    fn render_scene(&self, fb: &mut Framebuffer) {
        let w = fb.width as u32;
        let h = fb.height as u32;

        fill(fb, 0, 0, w, h, rgb(0x10, 0x12, 0x18));

        // Taskbar.
        fill(fb, 0, 0, w, BAR_H, rgb(0x22, 0x28, 0x36));
        text(fb, 6, 4, "all-de", rgb(0x7C, 0xF0, 0x9C));
        let mut tx: u32 = 64;
        for win in &self.windows {
            let label = if win.minimized {
                format!("[{}]", win.title)
            } else {
                win.title.clone()
            };
            text(fb, tx, 4, &label, rgb(0xD0, 0xD8, 0xE0));
            tx += (label.len() as u32 * 8) + 14;
        }

        // Windows.
        let visible = self.visible_indices();
        let n = visible.len().max(1) as u32;
        let col_w = w / n;
        for (i, &idx) in visible.iter().enumerate() {
            let x = i as u32 * col_w;
            self.render_window(fb, idx, x, BAR_H, col_w, h - BAR_H);
        }
    }

    fn render_window(&self, fb: &mut Framebuffer, idx: usize, x: u32, y: u32, w: u32, h: u32) {
        let win = &self.windows[idx];
        let focused = idx == self.focused;
        let border = if focused { rgb(0x7C, 0xF0, 0x9C) } else { rgb(0x3A, 0x44, 0x5A) };

        fill_rounded(fb, x, y, w, h, rgb(0x1E, 0x22, 0x2C));
        border_rounded(fb, x, y, w, h, border);

        fill(fb, x + 2, y + 2, w - 4, TITLE_H, rgb(0x2A, 0x35, 0x4A));
        text(fb, x + 6, y + 4, &win.title, rgb(0xD0, 0xD8, 0xE0));
        text(fb, x + w - 26, y + 4, "_", rgb(0xC8, 0xD0, 0xDC));
        text(fb, x + w - 14, y + 4, "x", rgb(0xC8, 0xD0, 0xDC));

        let max_lines = ((h.saturating_sub(TITLE_H + 8)) / 10).max(1) as usize;
        let lines = win.visible_lines(max_lines);
        let mut ty = y + TITLE_H + 8;
        for line in &lines {
            let color = if line.starts_with(PROMPT) { rgb(0x7C, 0xF0, 0x9C) } else { rgb(0xC8, 0xD0, 0xDC) };
            text(fb, x + 6, ty, line, color);
            ty += 10;
        }
    }
}

fn split_once_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

// --- drawing helpers ---

fn set(fb: &mut Framebuffer, x: u32, y: u32, c: u32) {
    let (w, h) = (fb.width, fb.height);
    let (x, y) = (x as usize, y as usize);
    if x >= w || y >= h {
        return;
    }

    // The console's global framebuffer flip (FLIP for Rgb888, FLIP_X for
    // Indexed8) is applied inside `Framebuffer::set`. Compensate for it here so
    // the desktop draws in a top-left origin, left-to-right orientation — the
    // same on-screen orientation the console produces for its own text.
    let flip_y = fb.format == PixelFormat::Rgb888;
    let flip_x = fb.format == PixelFormat::Indexed8;
    let px = if flip_x { w - 1 - x } else { x };
    let py = if flip_y { h - 1 - y } else { y };

    fb.set(px, py, c);
}

fn get(fb: &Framebuffer, x: u32, y: u32) -> u32 {
    let (w, h) = (fb.width, fb.height);
    let (x, y) = (x as usize, y as usize);
    if x >= w || y >= h {
        return 0;
    }
    let flip_y = fb.format == PixelFormat::Rgb888;
    let flip_x = fb.format == PixelFormat::Indexed8;
    let px = if flip_x { w - 1 - x } else { x };
    let py = if flip_y { h - 1 - y } else { y };
    fb.get(px, py)
}

fn fill(fb: &mut Framebuffer, x: u32, y: u32, w: u32, h: u32, c: u32) {
    let (fw, fh, stride, ptr, format) = (fb.width, fb.height, fb.stride, fb.ptr, fb.format);

    // Clip to the framebuffer.
    let x0 = x.min(fw as u32);
    let y0 = y.min(fh as u32);
    let x1 = (x + w).min(fw as u32);
    let y1 = (y + h).min(fh as u32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    let r = (c >> 16) & 0xFF;
    let g = (c >> 8) & 0xFF;
    let b = c & 0xFF;

    // Fast path: write directly to the framebuffer. Logical (x, y) maps to the
    // same physical pixel as `set`, so there is no per-pixel flip/bounds
    // overhead — this makes full-screen fills usable at 1920x1080.
    match format {
        PixelFormat::Rgb888 => {
            let word = (r << 16) | (g << 8) | b;
            for yy in y0..y1 {
                let row = yy as usize * stride;
                for xx in x0..x1 {
                    unsafe {
                        (ptr.add(row + xx as usize * 4) as *mut u32).write_volatile(word);
                    }
                }
            }
        }
        PixelFormat::Indexed8 => {
            let idx = (((r >> 5) & 0x7) << 5 | ((g >> 5) & 0x7) << 2 | ((b >> 6) & 0x3)) as u8;
            for yy in y0..y1 {
                let row = yy as usize * stride;
                for xx in x0..x1 {
                    unsafe {
                        ptr.add(row + xx as usize).write_volatile(idx);
                    }
                }
            }
        }
        PixelFormat::Planar4 => {
            for yy in y0..y1 {
                for xx in x0..x1 {
                    set(fb, xx, yy, c);
                }
            }
        }
    }
}

fn corner_dist(v: u32, len: u32) -> i32 {
    let v = v as i32;
    let len = len as i32;
    if v < RADIUS {
        RADIUS - v
    } else if v >= len - RADIUS {
        v - (len - RADIUS) + 1
    } else {
        0
    }
}

fn fill_rounded(fb: &mut Framebuffer, x: u32, y: u32, w: u32, h: u32, c: u32) {
    if w == 0 || h == 0 {
        return;
    }
    let r = RADIUS as u32;

    // Interior rows are full-width (no corner rounding) — fill them directly.
    if h > 2 * r {
        fill(fb, x, y + r, w, h - 2 * r, c);
    }

    // Only the top/bottom `r` rows can contain rounded corners.
    let top = r.min(h);
    for yy in 0..top {
        for xx in 0..w {
            let dx = corner_dist(xx, w);
            let dy = corner_dist(yy, h);
            if dx * dx + dy * dy <= RADIUS * RADIUS {
                set(fb, x + xx, y + yy, c);
            }
        }
        let by = h - 1 - yy;
        if by >= top {
            for xx in 0..w {
                let dx = corner_dist(xx, w);
                let dy = corner_dist(by, h);
                if dx * dx + dy * dy <= RADIUS * RADIUS {
                    set(fb, x + xx, y + by, c);
                }
            }
        }
    }
}

fn border_rounded(fb: &mut Framebuffer, x: u32, y: u32, w: u32, h: u32, c: u32) {
    for xx in RADIUS as u32..w - RADIUS as u32 {
        set(fb, x + xx, y, c);
        set(fb, x + xx, y + h - 1, c);
    }
    for yy in RADIUS as u32..h - RADIUS as u32 {
        set(fb, x, y + yy, c);
        set(fb, x + w - 1, y + yy, c);
    }
}

fn text(fb: &mut Framebuffer, x: u32, y: u32, s: &str, c: u32) {
    let mut cx = x;
    for &b in s.as_bytes() {
        if (32..=126).contains(&b) {
            let glyph = &FONT8X8[(b - 32) as usize];
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (0x80 >> col) != 0 {
                        set(fb, cx + col, y + row as u32, c);
                    }
                }
            }
        }
        cx += 8;
    }
}

/// Triangular pointer with the hot-spot at (x, y): white outline, dark fill.
fn draw_cursor(fb: &mut Framebuffer, x: u32, y: u32) {
    for yy in 0..CURSOR_H {
        for xx in 0..CURSOR_W {
            let diag = (yy as i32) >= (xx as i32) * (CURSOR_H as i32) / (CURSOR_W as i32);
            if !diag {
                continue;
            }
            let near_diag =
                (yy as i32) <= (xx as i32) * (CURSOR_H as i32) / (CURSOR_W as i32) + 1;
            let near_bottom = yy >= CURSOR_H - 2;
            let near_left = xx <= 1;
            let c = if near_diag || near_bottom || near_left {
                rgb(0xFF, 0xFF, 0xFF)
            } else {
                rgb(0x15, 0x15, 0x15)
            };
            set(fb, x + xx, y + yy, c);
        }
    }
}

/// Remember the scene pixels currently covered by the cursor so they can be
/// restored before the cursor moves (avoids re-rendering the whole desktop).
fn cursor_save(fb: &Framebuffer, x: u32, y: u32, buf: &mut [u32]) {
    for yy in 0..CURSOR_H {
        for xx in 0..CURSOR_W {
            buf[(yy * CURSOR_W + xx) as usize] = get(fb, x + xx, y + yy);
        }
    }
}

fn cursor_restore(fb: &mut Framebuffer, x: u32, y: u32, buf: &[u32]) {
    for yy in 0..CURSOR_H {
        for xx in 0..CURSOR_W {
            set(fb, x + xx, y + yy, buf[(yy * CURSOR_W + xx) as usize]);
        }
    }
}

/// Run the interactive desktop loop. Returns control to the terminal on ESC.
pub fn run() {
    crate::terminal::set_keycode_capture(true);
    unsafe { mouse_init(); }

    let fb = match crate::gfx::console::try_fb() {
        Some(f) => f,
        None => {
            crate::terminal::set_keycode_capture(false);
            return;
        }
    };

    let mut d = Desktop::new();
    let (sw, sh) = (fb.width as u32, fb.height as u32);

    // Render the scene once, then overlay the cursor. The pixels under the
    // cursor are remembered so that moving the cursor only touches that small
    // region instead of re-rendering the whole desktop.
    d.render_scene(&mut *fb);
    let mut cur_buf: Vec<u32> = Vec::new();
    cur_buf.resize((CURSOR_W * CURSOR_H) as usize, 0);
    let mut cur_x = d.cursor_x;
    let mut cur_y = d.cursor_y;
    cursor_save(&*fb, cur_x, cur_y, &mut cur_buf);
    draw_cursor(&mut *fb, cur_x, cur_y);

    let mut prev_buttons = 0i32;
    let max_x = sw.saturating_sub(CURSOR_W) as i32;
    let max_y = sh.saturating_sub(CURSOR_H) as i32;

    loop {
        let mut cursor_moved = false;
        let mut full = false;

        // Poll the mouse: move the cursor and click (focus) on button press.
        unsafe {
            let mut dx = 0i32;
            let mut dy = 0i32;
            let mut dz = 0i32;
            let mut buttons = 0i32;
            if mouse_poll(&mut dx, &mut dy, &mut dz, &mut buttons) == 1 {
                let nx = (d.cursor_x as i32 + dx).clamp(0, max_x) as u32;
                let ny = (d.cursor_y as i32 + dy).clamp(0, max_y) as u32;
                if nx != d.cursor_x || ny != d.cursor_y {
                    d.cursor_x = nx;
                    d.cursor_y = ny;
                    cursor_moved = true;
                }
                if (buttons & 1) != 0 && (prev_buttons & 1) == 0 {
                    d.click(sw, sh);
                    full = true;
                }
                prev_buttons = buttons;
            }
        }

        match crate::terminal::pop_keycode() {
            Some(k) => {
                if d.handle_keycode(k, sw, sh) {
                    break;
                }
                full = true;
            }
            None => {}
        }

        if full {
            // Content changed: restore the cursor region, re-render the scene,
            // then redraw the cursor on top.
            cursor_restore(&mut *fb, cur_x, cur_y, &cur_buf);
            d.render_scene(&mut *fb);
            cursor_save(&*fb, d.cursor_x, d.cursor_y, &mut cur_buf);
            cur_x = d.cursor_x;
            cur_y = d.cursor_y;
            draw_cursor(&mut *fb, cur_x, cur_y);
        } else if cursor_moved {
            // Only the cursor changed: restore the old spot and draw the new one.
            cursor_restore(&mut *fb, cur_x, cur_y, &cur_buf);
            cursor_save(&*fb, d.cursor_x, d.cursor_y, &mut cur_buf);
            cur_x = d.cursor_x;
            cur_y = d.cursor_y;
            draw_cursor(&mut *fb, cur_x, cur_y);
        } else {
            // Idle: keep polling. The PS/2 mouse driver is polled (not interrupt
            // driven), so this keeps the cursor responsive and loses no movement.
            core::hint::spin_loop();
        }
    }

    crate::terminal::set_keycode_capture(false);
    crate::vga_buffer::WRITER.lock().clear_screen();
    crate::gfx::refresh();
}

