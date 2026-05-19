# FH Language Combo Tool

FH Language Combo Tool helps you use different voice and text languages in Forza Horizon 5 / 6 (Steam, PC).

For example: English text + Japanese voice, or English text + Korean voice.

## Download

Download the latest version from the [Releases](https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases) page.

- **Portable**: `FH-Language-Combo-Tool-Portable.exe` — run directly, no installation needed
- **Installer**: `FH-Language-Combo-Tool-Setup.exe` — installs to your system

## Quick Start

1. Download and run the tool
2. Accept the disclaimer
3. The tool auto-detects your FH5/FH6 Steam installation
4. Select **Voice Language** — the language you want to HEAR (character voices)
5. Select **Text Language** — the language you want to SEE (menus, subtitles)
6. Click **Apply**
7. Confirm the operation
8. Launch the game — done!

## How It Works

The tool copies the text language file over the voice language file position in the game's `StringTables` directory, then automatically sets the game's startup language to your chosen voice language. The game loads voice audio from one language but displays text from another.

## Restore

Click **Restore Backup** at any time to undo all changes and return to the original state.

## After Game Updates

Game updates may reset your language configuration. If this happens, simply re-apply the same settings — it takes seconds.

## FAQ

**Q: Do I need to download language packs in Steam first?**
A: Yes. In Steam, right-click the game → Properties → Language, and ensure both your voice and text languages are downloaded.

**Q: Is this safe?**
A: The tool only modifies text resource files (`.zip` in `StringTables`). It does not touch executables, save files, or anti-cheat. All changes are backed up and reversible.

**Q: Will I get banned?**
A: This tool does not modify gameplay, network data, or anti-cheat components. It only changes local text files. However, use at your own risk.

## Disclaimer

This is an unofficial tool. Not affiliated with Playground Games, Turn 10 Studios, Xbox, or Microsoft. Use at your own risk.
