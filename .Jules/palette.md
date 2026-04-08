# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2025-03-01 - CLI Input Clearing
**Learning:** In CLI applications, users frequently expect standard terminal shortcuts like Ctrl+U to clear the input before the cursor, which is missing by default in custom raw mode input loops.
**Action:** When implementing custom `crossterm` input handlers, map `Ctrl+U` to clear the string slice before the cursor and use ANSI escape codes `\r\x1b[2K` to cleanly erase the terminal line and redraw the prompt to prevent ghost characters.
