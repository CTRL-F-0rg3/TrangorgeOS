//! A userspace shell terminal: a text buffer plus a small line-based command
//! interpreter. Each terminal is an independent process with its own state.

pub struct Shell {
    pub prompt: String,
    pub lines: Vec<String>,
    pub input: String,
    pub closed: bool,
}

impl Shell {
    pub fn new(prompt: &str) -> Self {
        let mut s = Self {
            prompt: prompt.to_string(),
            lines: Vec::new(),
            input: String::new(),
            closed: false,
        };
        s.lines.push("TrangorgeOS all-de shell".to_string());
        s.lines.push("type 'help' for commands".to_string());
        s
    }

    /// Feed one character into the terminal (handles enter / backspace).
    pub fn feed_char(&mut self, c: char) {
        if self.closed {
            return;
        }
        match c {
            '\n' => self.submit(),
            '\x08' => {
                self.input.pop();
            }
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => self.input.push(c),
            _ => {}
        }
    }

    /// Submit the current input line to the interpreter.
    pub fn submit(&mut self) {
        let line = std::mem::take(&mut self.input);
        self.lines.push(format!("{}{}", self.prompt, line));
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        let (cmd, rest) = split_once_space(trimmed);
        let out: Vec<String> = match cmd {
            "help" => vec![
                "commands: help echo clear uname exit".to_string(),
                "  echo <text>   print text".to_string(),
                "  clear         clear the screen".to_string(),
                "  uname         print system name".to_string(),
                "  exit          close this terminal".to_string(),
            ],
            "echo" => vec![rest.to_string()],
            "clear" => {
                self.lines.clear();
                vec![]
            }
            "uname" => vec!["TrangorgeOS all-de userspace shell".to_string()],
            "exit" => {
                self.closed = true;
                vec![]
            }
            _ => vec![format!("{}: command not found (try 'help')", cmd)],
        };
        self.lines.extend(out);
    }

    /// The last `max` lines plus the current prompt/input (for rendering).
    pub fn visible_lines(&self, max: usize) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        let start = self.lines.len().saturating_sub(max);
        for l in &self.lines[start..] {
            v.push(l.clone());
        }
        v.push(format!("{}{}", self.prompt, self.input));
        v
    }
}

fn split_once_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}
