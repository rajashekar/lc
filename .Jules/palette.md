# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.
## 2026-03-25 - Preserve Dynamic Input Prompts
**Learning:** When building custom interactive terminal multi-line input handlers, hardcoded redraw strings (like 'You: ') can destroy dynamically styled prompts containing ANSI color codes during Backspace or redrawing events.
**Action:** Store the initial dynamic prompt string in the component's state and reuse it explicitly for all terminal line repaints to preserve correct colors and styling.
