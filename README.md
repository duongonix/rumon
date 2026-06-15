# Rumon

Rumon is a modern file watcher and process runner with an interactive terminal
dashboard. It is designed to explain what changed, why a restart happened, and
what the configured command is doing now.

## Status

Rumon is currently in `0.0.0-dev`.

Current implementation focus:

- Phase 0: workspace foundation
- Phase 1: core nodemon-style runtime

## Workspace

The project is organized as a Cargo workspace with one crate per runtime
responsibility:

- `rumon-cli`
- `rumon-core`
- `rumon-config`
- `rumon-watch`
- `rumon-runner`
- `rumon-tui`
- `rumon-diff`
- `rumon-media`
- `rumon-hooks`
- `rumon-profiles`
- `rumon-plugins`
- `rumon-remote`
- `rumon-shared`

## Development

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

See `docs/` for architecture, roadmap, and subsystem specifications.
