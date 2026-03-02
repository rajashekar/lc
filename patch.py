import re

with open("src/utils/input.rs", "r") as f:
    content = f.read()

# Replace crossterm imports to include MoveRight
content = content.replace(
    "cursor::MoveLeft,",
    "cursor::{MoveLeft, MoveRight},"
)

# 1. KeyCode::Char(c)
char_pattern = r"""(KeyCode::Char\(c\) => \{\s*// Insert character at cursor position\s*)self\.current_line\.insert\(self\.cursor_pos,\s*c\);\s*self\.cursor_pos \+= 1;\s*// Print the character\s*print!\("\{\}", c\);\s*// Redraw the rest of the line if we inserted in the middle\s*let rest = &self\.current_line\[self\.cursor_pos\.\.\];"""

char_replacement = r"""\1let byte_offset = self.current_line.char_indices().nth(self.cursor_pos).map(|(i, _)| i).unwrap_or(self.current_line.len());
                self.current_line.insert(byte_offset, c);
                self.cursor_pos += 1;

                // Print the character
                print!("{}", c);

                // Redraw the rest of the line if we inserted in the middle
                let rest = &self.current_line[byte_offset + c.len_utf8()..];"""
content = re.sub(char_pattern, char_replacement, content)

# 2. KeyCode::Delete
delete_pattern = r"""(KeyCode::Delete => \{\s*)if self\.cursor_pos < self\.current_line\.len\(\) \{\s*// Remove character at cursor\s*self\.current_line\.remove\(self\.cursor_pos\);\s*// Print rest of line\s*let rest = &self\.current_line\[self\.cursor_pos\.\.\];"""

delete_replacement = r"""\1if self.cursor_pos < self.current_line.chars().count() {
                    // Remove character at cursor
                    let byte_offset = self.current_line.char_indices().nth(self.cursor_pos).map(|(i, _)| i).unwrap_or(self.current_line.len());
                    self.current_line.remove(byte_offset);

                    // Print rest of line
                    let rest = &self.current_line[byte_offset..];"""
content = re.sub(delete_pattern, delete_replacement, content)


# 3. KeyCode::Backspace
backspace_pattern = r"""(KeyCode::Backspace => \{\s*if self\.cursor_pos > 0 \{\s*// Remove character before cursor\s*)self\.current_line\.remove\(self\.cursor_pos - 1\);\s*self\.cursor_pos -= 1;\s*// Move cursor back to position of deleted char\s*print!\("\\x08"\);\s*// Print rest of line\s*let rest = &self\.current_line\[self\.cursor_pos\.\.\];"""

backspace_replacement = r"""\1let byte_offset = self.current_line.char_indices().nth(self.cursor_pos - 1).map(|(i, _)| i).unwrap_or(self.current_line.len());
                    self.current_line.remove(byte_offset);
                    self.cursor_pos -= 1;

                    // Move cursor back to position of deleted char
                    print!("\x08");

                    // Print rest of line
                    let rest = &self.current_line[byte_offset..];"""
content = re.sub(backspace_pattern, backspace_replacement, content)


# 4. KeyCode::Backspace pop prev line
pop_pattern = r"""(self\.cursor_pos = prev_line)\.len\(\);"""
pop_replacement = r"""\1.chars().count();"""
content = re.sub(pop_pattern, pop_replacement, content)

# 5. KeyCode::Right
right_pattern = r"""(KeyCode::Right => \{\s*if self\.cursor_pos < self\.current_line)\.len\(\)(\s*\{\s*self\.cursor_pos \+= 1;\s*print!\("\\x1b\[C"\); // Move cursor right\s*io::stdout\(\)\.flush\(\)\?;\s*\})"""
right_replacement = r"""\1.chars().count()\2"""
content = re.sub(right_pattern, right_replacement, content)

# 6. Home/End
home_end = r"""            KeyCode::Home => {
                if self.cursor_pos > 0 {
                    execute!(io::stdout(), MoveLeft(self.cursor_pos as u16))?;
                    self.cursor_pos = 0;
                }
                Ok(InputAction::Continue)
            }
            KeyCode::End => {
                let chars_count = self.current_line.chars().count();
                if self.cursor_pos < chars_count {
                    execute!(io::stdout(), MoveRight((chars_count - self.cursor_pos) as u16))?;
                    self.cursor_pos = chars_count;
                }
                Ok(InputAction::Continue)
            }"""

content = content.replace("_ => Ok(InputAction::Continue),", f"{home_end}\n            _ => Ok(InputAction::Continue),")


with open("src/utils/input.rs", "w") as f:
    f.write(content)
