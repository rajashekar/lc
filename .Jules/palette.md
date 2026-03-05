# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-10-24 - Navigation for Multiline CLI Inputs
**Learning:** Terminal inputs without built-in libraries often force users to use `Left`/`Right` keys continuously to reach the beginning or end of lines, slowing down edits on long inputs. Keyboard navigation should match expected OS behavior.
**Action:** Always implement `Home` and `End` keys in terminal-based UI inputs. When moving the cursor, calculate the distance in *characters* rather than *bytes* to properly support UTF-8 inputs, utilizing `crossterm::cursor::MoveLeft` and `MoveRight`.
