# Contributing

## Tech Stack

- **Backend**: Rust + Tauri 2
- **Frontend**: Vanilla HTML / CSS / JS
- **Build**: `cargo tauri dev` (development) / `cargo tauri build` (production)

## Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) (`cargo install tauri-cli --version "^2"`)
- Windows 10/11 with MSVC build tools

## Development

```
cargo tauri dev
```

## Tests

```
cd src-tauri
cargo test                  # unit + portable integration tests
cargo test -- --ignored     # additional tests requiring real FH5/FH6 Steam installation
```

## Project Structure

```
src-tauri/
  src/
    core/           # business logic modules
      game_detector.rs      # Steam registry/VDF, directory validation
      resource_scanner.rs   # StringTables scanning, SHA-256
      language_mapper.rs    # language code mapping, plan generation
      backup_manager.rs     # backup creation/listing/manifest
      apply_engine.rs       # atomic file replacement + rollback
      restore_engine.rs     # restore from backup
      steam_language.rs     # Steam/game language setting
      logger.rs             # local file logging
    commands/       # Tauri command layer (thin wrappers)
  tests/            # integration tests
src/                # vanilla HTML/CSS/JS frontend
```

## Release

Push a version tag to trigger CI:

```
git tag v1.0.0
git push origin v1.0.0
```

The workflow builds a standalone portable `.exe` and an NSIS installer, then publishes them as a GitHub Release.
