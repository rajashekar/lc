# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - CLI Input Styling Persistence
**Learning:** Terminal inputs using custom ANSI styled prompts (e.g. bold, colors) lose their styling if the input handler relies on a hardcoded string placeholder during terminal repaints (like after line deletions or line-wrapping).
**Action:** Always store the exact passed prompt string in the component's internal state (`self.prompt`) so that terminal repaints accurately reflect the original, formatted prompt text. Also ensure users receive familiar terminal visual feedback, such as `^C` when cancelling an action.
