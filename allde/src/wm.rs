//! A niri-inspired tiling window manager: windows are tiled horizontally in
//! workspaces, can be reordered / moved between workspaces, and are focused by
//! keyboard or pointer.

pub type WindowId = u32;

/// A rectangular region of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// A window owned by the desktop (its content is rendered by its app).
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(id: WindowId, title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            id,
            title: title.into(),
            width,
            height,
        }
    }
}

/// The tiling manager: a list of workspaces, each a horizontal strip of
/// window ids. The active workspace is tiled into equal columns.
pub struct Wm {
    pub screen_w: u32,
    pub screen_h: u32,
    pub workspaces: Vec<Vec<WindowId>>,
    pub active_ws: usize,
    pub focused: Option<WindowId>,
}

impl Wm {
    pub fn new(screen_w: u32, screen_h: u32) -> Self {
        Self {
            screen_w,
            screen_h,
            workspaces: vec![Vec::new()],
            active_ws: 0,
            focused: None,
        }
    }

    pub fn add(&mut self, id: WindowId) {
        self.workspaces[self.active_ws].push(id);
        self.focused = Some(id);
    }

    pub fn remove(&mut self, id: WindowId) {
        for ws in &mut self.workspaces {
            ws.retain(|&w| w != id);
        }
        if self.focused == Some(id) {
            self.focused = self.workspaces[self.active_ws].last().copied();
        }
    }

    /// Reorder the window one column to the left.
    pub fn move_left(&mut self, id: WindowId) {
        let ws = &mut self.workspaces[self.active_ws];
        if let Some(pos) = ws.iter().position(|&w| w == id) {
            if pos > 0 {
                ws.swap(pos, pos - 1);
            }
        }
    }

    /// Reorder the window one column to the right.
    pub fn move_right(&mut self, id: WindowId) {
        let ws = &mut self.workspaces[self.active_ws];
        if let Some(pos) = ws.iter().position(|&w| w == id) {
            if pos + 1 < ws.len() {
                ws.swap(pos, pos + 1);
            }
        }
    }

    /// Move a window to another workspace (creating it if needed) and focus it.
    pub fn move_to_workspace(&mut self, id: WindowId, ws_idx: usize) {
        while self.workspaces.len() <= ws_idx {
            self.workspaces.push(Vec::new());
        }
        for ws in &mut self.workspaces {
            ws.retain(|&w| w != id);
        }
        self.workspaces[ws_idx].push(id);
        self.active_ws = ws_idx;
        self.focused = Some(id);
    }

    pub fn focus(&mut self, id: WindowId) {
        if self.workspaces[self.active_ws].contains(&id) {
            self.focused = Some(id);
        }
    }

    pub fn focus_next(&mut self) {
        let ws = &self.workspaces[self.active_ws];
        if ws.is_empty() {
            return;
        }
        let next = match self.focused {
            Some(id) => {
                let pos = ws.iter().position(|&w| w == id).unwrap_or(0);
                ws[(pos + 1) % ws.len()]
            }
            None => ws[0],
        };
        self.focused = Some(next);
    }

    pub fn focus_prev(&mut self) {
        let ws = &self.workspaces[self.active_ws];
        if ws.is_empty() {
            return;
        }
        let next = match self.focused {
            Some(id) => {
                let pos = ws.iter().position(|&w| w == id).unwrap_or(0);
                ws[(pos + ws.len() - 1) % ws.len()]
            }
            None => ws[ws.len() - 1],
        };
        self.focused = Some(next);
    }

    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.workspaces[self.active_ws].iter().copied()
    }

    /// Compute the tiled layout of the active workspace: each window gets an
    /// equal horizontal column spanning the full height.
    pub fn layout(&self) -> Vec<(WindowId, Rect)> {
        let ws = &self.workspaces[self.active_ws];
        let n = ws.len().max(1) as u32;
        let col_w = self.screen_w / n;
        let h = self.screen_h;

        ws.iter()
            .enumerate()
            .map(|(i, &id)| {
                (
                    id,
                    Rect {
                        x: (i as u32 * col_w) as i32,
                        y: 0,
                        w: col_w,
                        h,
                    },
                )
            })
            .collect()
    }

    /// The window under the given screen coordinates, if any.
    pub fn window_at(&self, x: u32, y: u32) -> Option<WindowId> {
        for (id, r) in self.layout() {
            if x >= r.x as u32
                && x < (r.x as u32 + r.w)
                && y >= r.y as u32
                && y < (r.y as u32 + r.h)
            {
                return Some(id);
            }
        }
        None
    }
}
