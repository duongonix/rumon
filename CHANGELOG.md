# Changelog

All notable changes to Rumon will be documented in this file.

## 0.0.0-dev

- Initialized Cargo workspace foundation.
- Added crate boundaries for core Rumon subsystems.
- Added Phase 0 project metadata, default config, and CI workflow.
- Implemented Phase 1 CLI parsing, config resolution, event bus, polling watcher, and process runner.
- Added stdout/stderr capture, process restart support, and no-restart/run-once execution paths.
- Refactored crate `lib.rs` files to only declare modules and public exports.
- Implemented Phase 2 TUI state, layout, focus, keyboard commands, changes/logs panels, status bar, footer, search, help, and dashboard rendering.
- Integrated the TUI with watch mode as the default runtime dashboard, including alternate-screen rendering, live state redraws, restart/quit/clear-log commands, and `--no-tui` plain-log fallback.
- Reduced TUI flicker by skipping duplicate frame writes and drawing updated frames without clearing the full screen first.
- Batched pending runtime events before TUI redraws to reduce log-panel redraw churn during output bursts.
- Updated default watch roots to include `src`, `crates`, and `rumon.toml` so workspace crate edits trigger watch events.
- Fixed TUI log rendering so child stdout containing `restart` is not mistaken for a restart marker, ANSI control sequences are stripped, and restart counts come from runtime state.
- Changed this repository's `rumon.toml` run command to `cargo check --workspace` to avoid recursively running Rumon inside Rumon during local development.
- Added ANSI semantic styling, status/change/log icons, colored borders, highlighted footer keys, styled overlays, and ANSI-aware layout fitting for a more polished TUI.
- Added type-aware change details in the TUI, including seeded text diffs with line/column summaries, media metadata, binary size/hash summaries, and deleted-file details.
- Implemented Phase 3 diff engine with text detection, line diffs, inline/column changes, binary metadata hashing, and preview limits.
- Implemented Phase 4 media detection, image/audio/video metadata summaries, and metadata comparison.
- Implemented Phase 5 lifecycle hooks with hook points, environment injection, timeout handling, output capture, and log summaries.
- Implemented Phase 6 profiles with built-in rust/node/go/python/docker presets, custom `profiles/<name>.toml` loading, profile-before-user-config merge order, CLI override support, and multiline profile arrays.
- Implemented Phase 8 remote monitor out of order with TCP transport, token auth, agent/client commands, remote state/event/log/disconnect frames, and a TUI remote node panel renderer.
- Updated the TUI to use the actual terminal window size on Windows and Unix instead of relying only on `COLUMNS`/`LINES`, allowing the dashboard to expand to the full terminal.
- Extended the watcher to snapshot directories as well as files, emit created/modified/deleted/renamed events, detect file and folder renames, and show renamed paths in plain output and the TUI.
- Refined the TUI diff preview with Git-style line rows, colored added/removed row backgrounds, bold change markers, and stronger ANSI contrast.
- Reworked the TUI toward the reference design with a 58/42 split, focused panel borders, block-based change cards, fixed panel titles, per-panel navigation, Enter expand/collapse, concise log rows, footer navigation hints, and no hunk headers in diff previews.
- Switched TUI input to raw terminal key events so arrow keys, PageUp/PageDown, Home/End, Tab, and Enter scroll or toggle the focused panel without typing commands.
- Fixed doubled TUI key handling on terminals that emit key release events by processing only key press events.
- Moved the focused panel indicator from vertical borders to a single yellow title underline directly beneath the active Changes or Logs panel header.
- Added rule-based `[[event_hooks]]` config with event type filtering, normalized glob path matching, rename old/new path matching, hook env vars, and stdout/stderr logging into the TUI Logs panel.
- Added a cached mini rule engine for `event_hooks.when` expressions with boolean logic, comparisons, arithmetic, membership, string predicates, regex matching, size/duration units, syntax diagnostics, and verbose match/skip logs.
- Adjusted TUI change blocks to occupy about 90% of the Changes panel width and moved change status/stat text onto a second header line.
- Reworked TUI search and help overlays into centered dialogs with dedicated borders and fixed footer-safe layout space.
- Replaced the default watcher path with a native `notify` + `notify-debouncer-full` backend, using `ignore` for initial traversal and retaining polling as a fallback.
- Normalized watch paths to display relative to the Rumon working directory in plain logs, TUI changes, hook glob matching, and hook path environment variables.
- Simplified plain `--no-tui` output to hide routine process lifecycle noise while keeping command output and colored file change events.
- Polished the TUI logs panel by removing the command header, coloring stdout prompts, and replacing text expand/collapse markers with icon chevrons.
- Fixed TUI diff preview rows so modified lines are split into separate removed/added rows instead of embedding newline characters that could break card borders.
- Switched text line diffing from the hand-written LCS implementation to `similar`, while keeping Rumon's UI-facing diff model and Git/Codex-style TUI rendering.
- Restored Changes panel scrolling by switching the left panel viewport from change-index scrolling to rendered-line scrolling, including single large change cards.
- Migrated the active TUI screen backend to `ratatui` with `crossterm`, keeping existing app state and input logic while replacing full-frame ANSI writes with Ratatui buffer rendering.
- Cleaned up the Ratatui dashboard header rows by removing duplicate panel titles, using a smaller stdout prompt glyph, coloring change stats by sign, and adding diff row backgrounds.
- Fixed Ratatui panel scrolling so Changes/Logs headers stay pinned while content scrolls and diff preview line numbers start from the detected change location.
- Split the Ratatui renderer into focused modules and fixed change card header width so the right border aligns with the panel instead of appearing inset.
- Added `rumon init` to create a starter `rumon.toml` in the current project without overwriting an existing config.
- Added protocol integration foundations: `rumon watch --json/--ndjson`, serde event schema, NDJSON writer, shared protocol event bus, HTTP/SSE/WebSocket server mode, and NDJSON IPC daemon mode.
- Hardened integration modes with RFC3339 UTC timestamps, real `--json/--ndjson --once` watch collection, Axum/Tokio HTTP/SSE/WebSocket serving, native local-socket IPC via named pipes on Windows and Unix sockets on Unix, IPC shutdown, and realtime transport smoke coverage.
- Updated the TUI status/footer polish with a highlighted `0.1.0` version label, balanced 50/50 Changes/Logs panels, and brighter quit/clear-log footer labels.
- Reworked release deployment assets for Rumon, including GitHub Actions binary packaging, install scripts, and release tag helpers that update the workspace version.
- Added the Astro Starlight Rumon docs landing page with a dark cyberpunk cyan theme, modular hero/dashboard components, and global Starlight styling overrides.
- Made the Rumon docs landing page sidebar-free, forced the site to dark mode, and removed the Starlight theme switcher.
- Split the landing page into a standalone Astro home route so `/` no longer renders through the Starlight docs frame or sidebar.
- Improved docs sidebar active/hover contrast and replaced the home hero dashboard mockup with the real `public/rumon.png` product image.
- Expanded the Astro Starlight docs into a complete Rumon documentation set covering installation, quick start, CLI, watcher, diff engine, runner, profiles, config, hooks, rule engine, TUI, integrations, guides, API, and troubleshooting.
- Fixed Linux CI build by using `interprocess::os::unix::local_socket::FilesystemUdSocket` for Unix IPC filesystem socket names.
