# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-06-25 - Terminal Redraw on Clear Commands
**Learning:** Modifying strings or clearing lines (such as with `Ctrl+U`) within custom terminal input handlers like `MultiLineInput` requires manual redraws using ANSI escape sequences (e.g., `\r\x1b[2K`) and cursor resets. Otherwise, visual artifacts remain on screen.
**Action:** When adding line-clearing functionality, pass the original formatted `prompt` text down through the input handling chain so it can be cleanly redrawn after clearing the raw user input via terminal escape codes.
