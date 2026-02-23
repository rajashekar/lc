# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - Zero-Friction Onboarding
**Learning:** Users arriving at documentation often want to "install and run" immediately. Hiding installation steps in a sub-page increases friction.
**Action:** Always provide "Quick Install" tabs (e.g., Shell/Cargo) directly on the landing page (intro.md) to convert interest into action instantly.
