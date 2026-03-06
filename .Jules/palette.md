# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-25 - Multibyte Text Input Navigation
**Learning:** Manual terminal input implementations must treat string manipulation and cursor movement carefully when dealing with multibyte UTF-8 characters. Relying on byte indices for cursor tracking causes crashes when a user navigates and attempts to insert/delete emojis or other non-ASCII characters, as the byte index may point to the middle of a character boundary.
**Action:** Always decouple cursor tracking (which must represent character count) from string mutation indexing (which requires byte indices). Convert character indices to byte indices dynamically (e.g. using `.char_indices()`) before performing any string mutations.
