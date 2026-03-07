# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2023-10-24 - Interactive Repainting Styling Fix
**Learning:** Hardcoding prompt strings like "You:" during line repainting (e.g. after a backspace clear) causes loss of original terminal styling (like colors, bolding) since the terminal uses ANSI escape codes that need to be re-emitted.
**Action:** When creating reusable interactive UI elements in the CLI, store the original dynamic, styled prompt string in the state struct (`self.prompt`) and use it during repaints rather than hardcoded text placeholders to preserve consistent UX.
