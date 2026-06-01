1. **Fix `src/core/provider_installer.rs`**: Use `replace_with_git_merge_diff` to add the `is_safe_filename` validation and update `install_provider` and `uninstall_provider`.
2. **Format Code**: Run `cargo fmt`.
3. **Start tests**: Run `cargo test --all-targets --no-default-features --features "pdf,unix-sockets,s3-sync" > test_output.log 2>&1 &` in the background.
4. **Monitor tests**: Repeatedly run `cat test_output.log` to check the progress.
5. **Clean up**: Remove `test_output.log` with `rm test_output.log`.
6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
7. **Submit the PR**: Submit the changes with branch `sentinel-path-traversal` and title `🛡️ Sentinel: [HIGH] Fix path traversal in provider installer`, with a description including 🚨 Severity, 💡 Vulnerability, 🎯 Impact, 🔧 Fix, and ✅ Verification.
