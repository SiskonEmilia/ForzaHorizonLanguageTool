# FH Language Combo Tool

A local utility for configuring separate voice and text languages in Forza Horizon 5 / Forza Horizon 6 (Steam, PC).

## What it does

Forza Horizon games only allow switching all languages at once. This tool lets you use one language for voice (e.g. Japanese) and a different language for UI text / subtitles (e.g. Simplified Chinese), by safely copying the text language pack over the voice language pack's slot.

Every modification is preceded by an automatic backup and can be reversed with one click.

## Features

- Auto-detects Steam installations of FH5 and FH6
- Scans available language packs (24 per game)
- Generates and previews a file operation plan before making changes
- Creates a timestamped backup with SHA-256 verification before every modification
- One-click restore from any backup
- Detects whether a previous configuration is still active or has been reverted by a game update
- Checks if the game is running before applying changes
- Local operation log

## How it works

1. The game is set to the desired **voice language** via Steam / in-game settings.
2. The tool copies the desired **text language** pack (e.g. `CHS.zip`) over the voice language pack's file slot (e.g. `JP.zip`) in `media/Stripped/StringTables/`.
3. When the game loads, it reads the voice language's audio banks (Japanese voice) but the replaced text resource (Chinese text).

The tool only touches `.zip` files inside `StringTables`. It does not modify executables, save files, network data, anti-cheat components, or anything that affects gameplay fairness.

## Supported games

| Game | Steam App ID | Status |
|------|-------------|--------|
| Forza Horizon 5 | 1551360 | Supported |
| Forza Horizon 6 | 2483190 | Supported |

**Platform:** Windows 10 22H2+ / Windows 11  
**Distribution channel:** Steam only

## Tech stack

- **Backend:** Rust + Tauri 2
- **Frontend:** Vanilla HTML / CSS / JS (no framework)
- **Packaging:** Single executable via Tauri bundler

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) (`cargo install tauri-cli --version "^2"`)
- Windows with MSVC build tools

### Development

```
cd src-tauri
cargo tauri dev
```

### Production build

```
cd src-tauri
cargo tauri build
```

The installer will be in `src-tauri/target/release/bundle/`.

### Running tests

```
cd src-tauri
cargo test
```

## Disclaimer

This project is for educational and research purposes only.

This project is **not** an official tool of Forza, Xbox, Microsoft, Playground Games, or Turn 10 Studios.

This project does not provide, download, or crack any game resources. It does not bypass DRM or anti-cheat systems. It does not modify save files, account data, network data, or anything that affects online gameplay fairness.

Using this tool may cause game files to be modified. Game updates or file integrity verification may revert the changes. Use only on legally owned copies of the game, and at your own risk.

## License

[MIT](LICENSE)
