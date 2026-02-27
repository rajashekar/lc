# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - Empty State Guidance
**Learning:** Users often hit "dead ends" when running list commands on empty configurations, leaving them unsure of the next step.
**Action:** Always provide the command to create an item when displaying an empty list state. Use a "💡" icon to make it stand out as a tip.
