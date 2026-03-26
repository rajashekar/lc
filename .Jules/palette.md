# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-23 - Terminal Prompt ANSI Redraws
**Learning:** Hardcoding standard prefix strings (like `"You: "`) when redrawing multiline terminal prompts destroys any ANSI escape codes (colors, bolding) defined dynamically. Also, failing to provide explicit visual feedback upon input cancellation (like `^C`) breaks Unix conventions and degrades the UX.
**Action:** Always store and reuse the original dynamic prompt string during terminal state redraws instead of hardcoding placeholder text. Explicitly print standard visual feedback signals like `^C` to stdout when a user cancels an interactive prompt.
