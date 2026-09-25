//! Process / app registry: every app is a process that owns one window.

use crate::wm::WindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Shell,
    StatusBar,
    Info,
}

pub struct App {
    pub pid: u32,
    pub kind: AppKind,
    pub window: WindowId,
    pub title: String,
}

pub struct ProcessRegistry {
    next_pid: u32,
    apps: Vec<App>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            next_pid: 1,
            apps: Vec::new(),
        }
    }

    /// Spawn a process owning the given window; returns its pid.
    pub fn spawn(&mut self, kind: AppKind, window: WindowId, title: impl Into<String>) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.apps.push(App {
            pid,
            kind,
            window,
            title: title.into(),
        });
        pid
    }

    /// Kill a process; returns the window it owned.
    pub fn kill(&mut self, pid: u32) -> Option<WindowId> {
        let pos = self.apps.iter().position(|a| a.pid == pid)?;
        Some(self.apps.remove(pos).window)
    }

    pub fn by_window(&self, id: WindowId) -> Option<&App> {
        self.apps.iter().find(|a| a.window == id)
    }

    pub fn by_window_mut(&mut self, id: WindowId) -> Option<&mut App> {
        self.apps.iter_mut().find(|a| a.window == id)
    }

    pub fn remove_by_window(&mut self, id: WindowId) -> Option<App> {
        let pos = self.apps.iter().position(|a| a.window == id)?;
        Some(self.apps.remove(pos))
    }

    pub fn list(&self) -> Vec<(u32, String)> {
        self.apps
            .iter()
            .map(|a| (a.pid, a.title.clone()))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.apps.len()
    }
}

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self::new()
    }
}
