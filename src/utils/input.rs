use anyhow::Result;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
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
            eprintln!("Warning: Failed to enable raw mode: {}. Falling back to simple input.", e);
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
                // Insert character at cursor position
                self.current_line.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                
                // Print the character
                print!("{}", c);
                io::stdout().flush()?;
                
                Ok(InputAction::Continue)
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    // Remove character before cursor
                    self.current_line.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    
                    // Move cursor back, print space to clear character, move back again
                    print!("\x08 \x08");
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
                    print!("\x08"); // Move cursor left
                    io::stdout().flush()?;
                }
                Ok(InputAction::Continue)
            }
            KeyCode::Right => {
                if self.cursor_pos < self.current_line.len() {
                    self.cursor_pos += 1;
                    print!("\x1b[C"); // Move cursor right
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