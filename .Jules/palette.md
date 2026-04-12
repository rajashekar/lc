# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - Interactive CLI Input UX Improvements
**Learning:** Hardcoding generic strings like "You: " deep inside CLI input components prevents callers from effectively styling dynamic interactive prompts. Furthermore, implementing CLI editing commands like `Ctrl+U` or multi-line `Backspace` requires wiping current terminal lines using ANSI clear line sequences (`\r\x1b[2K`) before redrawing to prevent trailing character artifacts.
**Action:** When creating raw terminal input handlers (using raw mode in crossterm), propagate dynamic `prompt: &str` configurations down to internal redrawing logic. Always clear the entire line explicitly using ANSI sequences before replacing visual input state, avoiding spacing-based hacks (`" ".repeat(10)`).
