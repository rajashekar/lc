## 2024-05-18 - Passing Context to Terminal Redraws
**Learning:** Hardcoding string literals like `You: ` in raw mode terminal input handlers drops ANSI color code formatting when clearing and redrawing lines on user actions like `Backspace` or `Ctrl+U`.
**Action:** Always pass the dynamically formatted `prompt` string through to raw event handlers instead of recreating the text inline to ensure styling from crates like `colored` is preserved during screen updates.
