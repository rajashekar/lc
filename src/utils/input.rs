use anyhow::Result;
use colored::Colorize;
use crossterm::{
    cursor::MoveLeft,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

/// Multi-line input handler that supports Shift+Enter for new lines
pub struct MultiLineInput {
    lines: Vec<String>,
    current_line: String,
    cursor_pos: usize,
}

impl MultiLineInput {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_line: String::new(),
            cursor_pos: 0,
        }
    }

    /// Read multi-line input from the terminal
    /// - Enter: Submit the input
    /// - Shift+Enter: Add a new line
    /// - Ctrl+C: Cancel input (returns empty string)
    /// - Backspace: Delete character
    /// - Arrow keys: Navigate (basic support)
    pub fn read_input(&mut self, prompt: &str) -> Result<String> {
        print!("{} ", prompt);
        io::stdout().flush()?;

        // Ensure we can enable raw mode
        if let Err(e) = enable_raw_mode() {
            eprintln!(
                "Warning: Failed to enable raw mode: {}. Falling back to simple input.",
                e
            );
            return self.fallback_input();
        }

        let result = self.read_input_raw();

        // Always disable raw mode, even if there was an error
        let _ = disable_raw_mode();

        result
    }

    fn read_input_raw(&mut self) -> Result<String> {
        loop {
            let event = event::read()?;
            if let Event::Key(key_event) = event {
                // Only process key press events, ignore key release
                if key_event.kind == KeyEventKind::Press {
                    match self.handle_key_event(key_event)? {
                        InputAction::Continue => continue,
                        InputAction::Submit => {
                            // Add current line to lines if it's not empty
                            if !self.current_line.is_empty() {
                                self.lines.push(self.current_line.clone());
                            }

                            // Join all lines and return
                            let result = self.lines.join("\n");

                            // Clear state for next use
                            self.lines.clear();
                            self.current_line.clear();
                            self.cursor_pos = 0;

                            println!(); // Move to next line after input
                            return Ok(result);
                        }
                        InputAction::Cancel => {
                            // Clear state and return empty string
                            self.lines.clear();
                            self.current_line.clear();
                            self.cursor_pos = 0;

                            println!(); // Move to next line
                            return Ok(String::new());
                        }
                        InputAction::NewLine => {
                            // Add current line to lines and start a new line
                            self.lines.push(self.current_line.clone());
                            self.current_line.clear();
                            self.cursor_pos = 0;

                            // Print newline and show continuation prompt at beginning of line
                            print!("\r\n{}   ", "...".dimmed());
                            io::stdout().flush()?;
                        }
                    }
                }
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<InputAction> {
        // Debug: print key event details
        if std::env::var("LC_DEBUG_INPUT").is_ok() {
            eprintln!("[DEBUG] Key event: {:?}", key_event);
        }
        match key_event.code {
            KeyCode::Enter => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Enter: New line
                    Ok(InputAction::NewLine)
                } else {
                    // Enter: Submit
                    Ok(InputAction::Submit)
                }
            }
            KeyCode::Char('j') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+J: Alternative for new line (common in some terminals)
                Ok(InputAction::NewLine)
            }
            KeyCode::Char('\n') => {
                // Direct newline character (some terminals send this for Shift+Enter)
                Ok(InputAction::NewLine)
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C: Cancel
                Ok(InputAction::Cancel)
            }
            KeyCode::Char(c) => {
                // Determine byte position from char cursor_pos
                let byte_idx = self
                    .current_line
                    .chars()
                    .take(self.cursor_pos)
                    .map(|ch| ch.len_utf8())
                    .sum();

                // Insert character at correct byte position
                self.current_line.insert(byte_idx, c);
                self.cursor_pos += 1;

                // Print the character
                print!("{}", c);

                // Redraw the rest of the line if we inserted in the middle
                let rest_byte_idx = byte_idx + c.len_utf8();
                let rest = &self.current_line[rest_byte_idx..];
                if !rest.is_empty() {
                    print!("{}", rest);
                    // Move cursor back to correct position
                    let width = rest.chars().count() as u16;
                    execute!(io::stdout(), MoveLeft(width))?;
                }

                io::stdout().flush()?;
                Ok(InputAction::Continue)
            }
            KeyCode::Delete => {
                let char_count = self.current_line.chars().count();
                if self.cursor_pos < char_count {
                    // Determine the character to remove and its byte position
                    let mut char_to_remove_idx = 0;
                    for (current_pos, (i, _)) in self.current_line.char_indices().enumerate() {
                        if current_pos == self.cursor_pos {
                            char_to_remove_idx = i;
                            break;
                        }
                    }

                    // Remove character at cursor
                    self.current_line.remove(char_to_remove_idx);

                    // Print rest of line
                    let rest = &self.current_line[char_to_remove_idx..];
                    print!("{}", rest);

                    // Clear the character that was shifted left
                    print!(" ");

                    // Move cursor back to correct position
                    // We printed rest (len) + space (1)
                    let width = rest.chars().count() as u16 + 1;
                    execute!(io::stdout(), MoveLeft(width))?;

                    io::stdout().flush()?;
                }
                Ok(InputAction::Continue)
            }
            KeyCode::Backspace => {
                let char_count = self.current_line.chars().count();
                if self.cursor_pos > 0 && self.cursor_pos <= char_count {
                    // Determine the character to remove and its byte position
                    let mut char_to_remove_idx = 0;
                    for (current_pos, (i, _ch)) in self.current_line.char_indices().enumerate() {
                        if current_pos == self.cursor_pos - 1 {
                            char_to_remove_idx = i;
                            break;
                        }
                    }

                    // Remove character before cursor
                    self.current_line.remove(char_to_remove_idx);
                    self.cursor_pos -= 1;

                    // Move cursor back to position of deleted char visually
                    execute!(io::stdout(), crossterm::cursor::MoveLeft(1))?;

                    // Print rest of line
                    let rest_byte_idx = char_to_remove_idx;
                    let rest = &self.current_line[rest_byte_idx..];
                    print!("{}", rest);

                    // Clear the character that was shifted left
                    print!(" ");

                    // Move cursor back to correct position
                    // We printed rest (len) + space (1)
                    let width = rest.chars().count() as u16 + 1;
                    execute!(io::stdout(), MoveLeft(width))?;

                    io::stdout().flush()?;
                } else if !self.lines.is_empty() {
                    // If at beginning of current line and there are previous lines,
                    // move to end of previous line
                    let prev_line = self.lines.pop().unwrap();

                    // Clear current line display
                    print!("\r{}   \r", " ".repeat(10));

                    // Restore previous line
                    self.cursor_pos = prev_line.len();
                    self.current_line = prev_line;

                    // Redraw prompt and current line
                    if self.lines.is_empty() {
                        print!("You: {}", self.current_line);
                    } else {
                        print!("...   {}", self.current_line);
                    }
                    io::stdout().flush()?;
                }

                Ok(InputAction::Continue)
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    execute!(io::stdout(), crossterm::cursor::MoveLeft(1))?;
                    io::stdout().flush()?;
                }
                Ok(InputAction::Continue)
            }
            KeyCode::Right => {
                let char_count = self.current_line.chars().count();
                if self.cursor_pos < char_count {
                    self.cursor_pos += 1;
                    execute!(io::stdout(), crossterm::cursor::MoveRight(1))?;
                    io::stdout().flush()?;
                }
                Ok(InputAction::Continue)
            }
            _ => Ok(InputAction::Continue),
        }
    }

    /// Fallback to simple input when raw mode fails
    fn fallback_input(&mut self) -> Result<String> {
        eprintln!("Multi-line input (Shift+Enter) will not be available.");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}

#[derive(Debug)]
enum InputAction {
    Continue,
    Submit,
    Cancel,
    NewLine,
}

impl Default for MultiLineInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiline_input_creation() {
        let input = MultiLineInput::new();
        assert!(input.lines.is_empty());
        assert!(input.current_line.is_empty());
        assert_eq!(input.cursor_pos, 0);
    }
}
