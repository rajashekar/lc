## 2024-06-06 - Enhanced Embedded Chat Accessibility
**Learning:** For dynamic webchat interfaces embedded in Rust CLI applications, dynamically injected DOM elements (like chat messages) are completely invisible to screen readers unless the container explicitly opts-in to live region announcements.
**Action:** Always apply `aria-live="polite"` and `aria-atomic="false"` to dynamically updated chat containers to ensure new messages are announced without interrupting the user, and explicitly label all form inputs even if placeholders exist.
