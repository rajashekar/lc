# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-23 - Multi-byte Character Support in Terminal Inputs
**Learning:** Assuming a 1:1 mapping between character position and byte index in a manual terminal input implementation breaks text editing for multi-byte characters (e.g., emojis, international characters). This causes the program to crash or corrupt text when backspacing or inserting.
**Action:** When managing terminal input cursors, explicitly track the cursor position as a character index and map it back to byte indices (via `.char_indices()`) before performing any string manipulation (insert/remove) in Rust.
