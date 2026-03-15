# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2025-03-15 - [Multi-line Input UX and Stability]
**Learning:** Terminal inputs managing multi-line text with explicit cursor positions in Rust using `crossterm` can instantly crash if character-based cursors are used directly as byte indices in `String::insert` or `String::remove` operations. This happens frequently when users input multi-byte UTF-8 characters (like emojis or accents). Additionally, hardcoding redraw prompts (e.g. `"You: "`) strips ANSI escape codes (like colors and bolding), breaking visual consistency during edits like backspaces.
**Action:** When implementing custom text input handlers, always explicitly translate visual/character cursor positions to valid UTF-8 byte indices using `.char_indices()` before manipulating the underlying `String`. Ensure dynamically styled strings (like prompts) are preserved in state to maintain visual consistency during terminal repaints. Always print visual feedback (like `^C`) upon cancellation to align with standard Unix CLI expectations.
