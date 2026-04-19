# Palette's Journal

## 2024-05-23 - CLI Input UX
**Learning:** Manual implementation of multiline input in CLI tools often lacks basic editing features like inserting or deleting in the middle of a line.
**Action:** When implementing custom input handlers, always ensure that modifying the string in the middle triggers a redraw of the trailing characters and a cursor reset to the correct position. Use `crossterm`'s `MoveLeft` or similar ANSI codes to handle this efficiently.

## 2024-05-24 - Empty State Guidance
**Learning:** Users often hit "dead ends" when running list commands on empty configurations, leaving them unsure of the next step.
**Action:** Always provide the command to create an item when displaying an empty list state. Use a "💡" icon to make it stand out as a tip.

## 2024-05-24 - Consistent List Ordering
**Learning:** Inconsistent sorting between similar commands (e.g., global list vs. provider-specific list) confuses users and makes the tool feel unpolished.
**Action:** Always verify that all list-based outputs are sorted deterministically (e.g., alphabetically by ID) across all commands.

## 2024-05-23 - [Documentation DX Patterns]
**Learning:** Mixing alternative commands (e.g., aliases) into a single code block with "or" text breaks copy-paste functionality and frustrates users.
**Action:** Use interactive tabs or separate, clean code blocks for command variations to improve discoverability and reduce friction.

## 2024-05-25 - Empty State Command Hints
**Learning:** Returning a plain "No [items] found" message leaves users without clear next steps and increases friction. Providing the exact command to create an item improves discoverability.
**Action:** When implementing list commands that can return empty states, always display actionable guidance with the specific command to create a new item, ideally highlighted with an icon like '💡'.
