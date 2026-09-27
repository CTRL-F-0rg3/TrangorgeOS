//! A command terminal: a scrolling text buffer and a line editor.
//!
//! This is the part that owns *characters*. It has no idea what a command
//! means — it collects a line and hands it to a callback. That is the reason
//! it is separate from [`crate::apps::shell`]: the shell decides what `ls`
//! means, this decides how the answer gets typed and displayed.
//!
//! # Line editing
//!
//! Backspace, and a cursor that can move within the line. Deliberately not
//! more: history, tab completion and command-line editing belong to the
//! *shell*, because they are about commands rather than text. A terminal that
//! implemented them would be guessing at something it does not own.

/// One rendered line of output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    /// Draw in the prompt colour: the echoed input and the live prompt.
    pub is_prompt: bool,
}

impl Line {
    pub fn out(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_prompt: false,
        }
    }

    pub fn prompt(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_prompt: true,
        }
    }
}

/// A scrolling text buffer with a line editor.
#[derive(Debug, Clone)]
pub struct Terminal {
    lines: Vec<Line>,
    input: String,
    /// Cursor position within `input`, in bytes.
    cursor: usize,
    /// How many lines to keep; older ones are dropped once exceeded.
    capacity: usize,
    closed: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new(200)
    }
}

impl Terminal {
    /// A terminal keeping at most `capacity` lines.
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: Vec::new(),
            input: String::new(),
            cursor: 0,
            // Zero would drop every line, the prompt included.
            capacity: capacity.max(1),
            closed: false,
        }
    }

    /// Print one line of output.
    ///
    /// Takes the line, unlike [`Self::println`], so a `format!` at the call
    /// site is the common case and does not need a second method.
    pub fn print(&mut self, text: impl Into<String>) {
        self.lines.push(Line::out(text));
        self.trim();
    }

    pub fn print_prompt(&mut self, text: impl Into<String>) {
        self.lines.push(Line::prompt(text));
        self.trim();
    }

    /// Print an empty line.
    pub fn println(&mut self) {
        self.print("");
    }

    /// Print one line of output.
    pub fn say(&mut self, text: impl Into<String>) {
        self.print(text);
    }

    fn trim(&mut self) {
        if self.lines.len() > self.capacity {
            let excess = self.lines.len() - self.capacity;
            self.lines.drain(0..excess);
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Whether the user asked to leave.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Clear the scrollback, keeping the current input.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Feed one character from the keyboard.
    ///
    /// Returns the completed line when the user pressed Enter, so the caller
    /// can dispatch it. Ignored once the terminal is closed — a closed terminal
    /// that still accepted input is how a shell keeps running after `exit`.
    pub fn feed_char(&mut self, c: char) -> Option<String> {
        if self.closed {
            return None;
        }
        match c {
            '\n' | '\r' => {
                let line = std::mem::take(&mut self.input);
                self.cursor = 0;
                Some(line)
            }
            '\x08' | '\x7F' => {
                // Backspace removes the character *before* the cursor, not the
                // last one: with the cursor mid-line, deleting the last
                // character would make the cursor position meaningless.
                if self.cursor > 0 {
                    let i = self.prev_char_boundary();
                    self.input.remove(i);
                    self.cursor = i;
                }
                None
            }
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => {
                // A multi-byte character advances the cursor by its whole
                // length; one byte would land inside it, and the next
                // backspace would then split the character.
                let i = self.cursor;
                self.input.insert(i, c);
                self.cursor = i + c.len_utf8();
                None
            }
            _ => None,
        }
    }

    /// Move the cursor one character left.
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    /// Move the cursor one character right.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
            while self.cursor < self.input.len() && !self.input.is_char_boundary(self.cursor) {
                self.cursor += 1;
            }
        }
    }

    /// The byte index of the character before the cursor.
    fn prev_char_boundary(&self) -> usize {
        let mut i = self.cursor - 1;
        while i > 0 && !self.input.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// The lines to draw, ending with the live prompt and input.
    ///
    /// `max` is the number of text rows available. Returns the *last* `max`
    /// lines: a terminal shows the most recent output, and one scrolled to the
    /// top would be useless.
    pub fn visible_lines(&self, prompt: &str, max: usize) -> Vec<Line> {
        let mut v: Vec<Line> = Vec::with_capacity(max + 1);
        let start = self.lines.len().saturating_sub(max);
        v.extend_from_slice(&self.lines[start..]);
        v.push(Line::prompt(format!("{prompt}{}", self.input)));
        v
    }

    /// Every completed line, for a test or a serial dump.
    pub fn history(&self) -> Vec<&str> {
        self.lines.iter().map(|l| l.text.as_str()).collect()
    }
}
