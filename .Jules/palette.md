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
## 2024-05-24 - Empty State Guidance (Extended)
**Learning:** Found additional commands (aliases, templates, mcp, search providers, vectors) that lacked clear, actionable empty state guidance. Adding standard "💡" prompts is essential to prevent user dead ends.
**Action:** Always provide the command to create an item when displaying an empty list state. Use a "💡" icon to make it stand out as a tip, ensuring consistency across all CLI output.

## 2024-06-15 - Preserving Formatting in Embedded Web UIs
**Learning:** Simple HTML/JS chat interfaces for LLMs often fail to format text correctly because `div` text content naturally collapses whitespace and line breaks. This causes lists and code blocks from the model to become unreadable blobs of text.
**Action:** When building lightweight web UIs to display LLM output, always add `white-space: pre-wrap;` and `word-wrap: break-word;` to the message container's CSS to respect the model's original formatting while still wrapping long lines.
