1. Modify `src/cli/webchatproxy.rs`:
   - Change the button text from "Send" to indicate loading state when a request is active (e.g. changing it to "Sending...").
   - Update `sendMessage()` in the JS script to update the button's textContent.
   - Specifically: `button.textContent = 'Sending...';` right after `button.disabled = true;`, and `button.textContent = 'Send';` in the `finally` block before `button.disabled = false;`.
   - Also add `aria-label="Send message"` to the button for accessibility.

2. Run tests to ensure nothing is broken.
3. Complete pre-commit steps.
4. Submit the PR for the UX improvement.
