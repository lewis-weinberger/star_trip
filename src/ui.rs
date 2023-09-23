use crate::constants::*;

/// A terminal-like display offering a command-line
/// interface
pub struct Terminal {
    buffer: [u8; WIDTH * HEIGHT],
    history: [[u8; WIDTH - CWIDTH]; HIST],
    line: usize,
    cursor: usize,
}

impl Terminal {
    /// Creates a new instance with blank display
    pub fn new() -> Self {
        Self {
            buffer: [0; WIDTH * HEIGHT],
            history: [[32u8; WIDTH - CWIDTH]; HIST],
            line: 0,
            cursor: 0,
        }
    }

    /// Returns a pointer to the display buffer
    pub fn screen(&self) -> *const u8 {
        self.buffer[..SCREEN].as_ptr()
    }

    /// Returns a pointer to the command-line portion of the display buffer
    pub fn console(&self) -> *const u8 {
        self.buffer[SCREEN..].as_ptr()
    }

    /// Receives input, assuming code-page 437 encoding.
    /// Note: CP437 matches ASCII for printable characters.
    pub fn input(&mut self, c: u16) {
        if c < 256 {
            self.history[self.line][self.cursor] = c as u8;
            self.right();
        }
    }

    /// Prints the command-line to the display buffer
    pub fn update_console(&mut self) {
        self.buffer[SCREEN..(SCREEN + CWIDTH)].clone_from_slice(COMMAND);
        self.buffer[(SCREEN + CWIDTH)..].clone_from_slice(&self.history[self.line]);
        self.buffer[SCREEN + CWIDTH + self.cursor] = 219;
    }

    /// Moves the command-line input cursor one character to the left
    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.update_console();
        }
    }

    /// Moves the command-line input cursor one character to the right
    pub fn right(&mut self) {
        if self.cursor < WIDTH - CWIDTH - 1 {
            self.cursor += 1;
            self.update_console();
        }
    }

    /// Cycles forward through the command-line history
    pub fn down(&mut self) {
        self.cursor = 0;
        self.line = (self.line + 1) % HIST;
        self.update_console();
    }

    /// Cycles backward through the command-line history
    pub fn up(&mut self) {
        self.cursor = 0;
        self.line = (self.line - 1) % HIST;
        self.update_console();
    }

    /// Consumes full line of input as a single command
    pub fn enter(&mut self) -> [u8; WIDTH - CWIDTH] {
        let ret = self.history[self.line];
        self.down();
        self.history[self.line].copy_from_slice(&[32u8; WIDTH - CWIDTH]);
        ret
    }

    /// Print the message to the display, accounting for
    /// newlines and truncating any long lines
    pub fn message(&mut self, msg: &[u8]) {
        let mut text = msg.iter();
        for i in 0..HEIGHT {
            let mut j = 0;
            for &c in text.by_ref() {
                if c == b'\n' {
                    break;
                }

                // Truncate line beyond display width
                if j < WIDTH {
                    self.buffer[i * WIDTH + j] = c;
                }

                j += 1;
            }

            // Blank out rest of line after newline
            while j < WIDTH {
                self.buffer[i * WIDTH + j] = 0;
                j += 1;
            }
        }
    }
}
