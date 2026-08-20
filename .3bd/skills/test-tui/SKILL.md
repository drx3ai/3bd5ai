---
name: test-tui
description: Guide for testing Bd3 TUI interactively
---

You can start and use Bd3 TUI to verify changes. 

Important notes:

Start interactively.
Always set RUST_LOG="trace" when starting the process.
Pass `-c log_dir=<some_temp_dir>` argument to have logs written to a specific directory to help with debugging.
When sending a test message programmatically, send text first, then send Enter in a separate write (do not send text + Enter in one burst).
Use `just bd3` target to run - `just bd3 -c ...`
