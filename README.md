# GhostZip

GhostZip is a Windows-first desktop archive extractor built with Rust, Tauri, React, and TypeScript. Its main goal is to help extract archives on drives with tight free space by inspecting the archive first, checking target-drive space, and extracting files sequentially instead of blindly unpacking everything at once.

The repo now has two user-facing surfaces:

- `npm run dev` starts the public landing page for download, features, install steps, and deployment copy.
- `npm run tauri dev` starts the desktop extractor app.

## Verified Status

I checked the current repository behavior and verified:

- `cargo test --lib --bins` passes: 7 Rust unit tests passed.
- `npm run build` passes: TypeScript and Vite production build completed.
- The app has a normal extraction path and a destructive Extract & Purge path.
- ZIP Extract & Purge attempts to reclaim archive space during extraction on Windows by marking the ZIP as sparse and punching holes in compressed file data after each file is safely written.
- TAR, TAR.GZ, TAR.BZ2, TAR.XZ, 7Z, and RAR purge mode currently fall back to extracting first, then deleting the source archive at the end.

The full `cargo test` command compiled successfully but hit a doctest harness `BrokenPipe` while listing doctests. The actual unit-test run with `cargo test --lib --bins` passed cleanly.

## How It Works

1. Pick an archive and an output folder.
2. GhostZip detects the archive format from magic bytes where possible.
3. GhostZip inspects entries and builds a manifest with file count, directory count, compressed size, total uncompressed size, and target-drive free space.
4. GhostZip recommends space-saver behavior when available free space is less than or equal to the total uncompressed archive size.
5. You choose either standard extraction or Zero-Space Extract/Purge.
6. GhostZip writes each file through a temporary `.ghostzip-partial` file and renames it only after the write succeeds.
7. On completion, the extracted files are left in the selected output folder.

By default, the normal output folder is created beside the archive and named after the archive stem. For example, `Photos.zip` extracts to a `Photos` folder beside `Photos.zip`. Explorer's "Extract here" action instead uses the archive's parent folder directly.

## Does It Replace The Zipped File With Extracted Files?

Not exactly. GhostZip does not transform one file in place into a folder. It extracts the contents to the selected output directory.

Depending on the mode/settings:

- Standard Extract writes the extracted files to the output directory.
- The current UI has "Auto-delete source archive" enabled by default, so after a standard extraction completes, the completion screen automatically calls the delete command unless that setting is turned off.
- Zero-Space Extract/Purge deletes the source archive after extraction finishes.
- ZIP purge mode also tries to free disk allocation from the ZIP during extraction before the final delete.

So the end result can be: archive gone, extracted files present beside it or in the selected folder. That is controlled by extraction mode and the auto-delete setting.

## Space Behavior

GhostZip is space-aware, but it is not magic and it cannot extract with literally zero free bytes.

Standard Extract:

- Checks free space before files are written.
- Pauses if the target drive does not have enough room for the current file plus a 32 MiB safety floor.
- Leaves the archive untouched during extraction.

Zero-Space Extract/Purge for ZIP:

- Extracts one file at a time.
- Verifies the written file size and CRC32.
- Uses Windows sparse-file hole punching to try to release the compressed bytes for that ZIP entry after the file is extracted.
- Deletes the original ZIP at the end.
- Still needs enough free space for the current output file plus the 32 MiB safety floor.
- Works best on NTFS/ReFS-style Windows filesystems that support sparse files.

Zero-Space Extract/Purge for 7Z, RAR, and TAR variants:

- Currently extracts first.
- Deletes the source archive after the extraction succeeds.
- Does not currently reclaim source archive space file-by-file.

## Supported Formats

| Format | Detection | Inspection | Standard Extraction | Extract & Purge |
| --- | --- | --- | --- | --- |
| ZIP | Magic bytes | Yes | Yes | File-by-file sparse purge on Windows, then delete |
| 7Z | Magic bytes | Yes | Yes | Extract first, then delete |
| RAR | Magic bytes | Yes via bundled 7za helper | Yes via bundled 7za helper | Extract first, then delete |
| TAR | Magic bytes / extension fallback | Yes | Yes | Extract first, then delete |
| TAR.GZ | Magic bytes | Yes | Yes | Extract first, then delete |
| TAR.BZ2 | Magic bytes | Yes | Yes | Extract first, then delete |
| TAR.XZ | Magic bytes | Yes | Yes | Extract first, then delete |

## Safety Checks

GhostZip validates archive paths before writing:

- Absolute paths are rejected.
- `..` traversal is rejected.
- Output parents are canonicalized and checked against the canonical output directory.
- TAR symlinks are skipped.
- Partial output files use `.ghostzip-partial` and are removed on read/write/control errors where possible.
- ZIP purge mode verifies file size and CRC32 before purging that entry's compressed data.

## Build From Source

```powershell
npm install
npm run dev
npm run tauri dev
npm run build
npm run tauri build
```

The configured Windows bundle target is NSIS. Tauri writes the installer under `src-tauri/target/release/bundle/nsis/`.

If Windows reports `ENOENT` while tools try to create temporary files or the `dist` folder, move the repo to a normal writable development path such as `C:\dev\Ghostzip` or allow the folder in Windows Security Controlled Folder Access.

For release and hosting steps, see [DEPLOYMENT.md](DEPLOYMENT.md).

## Test From Source

```powershell
cd src-tauri
cargo test --lib --bins
cd ..
npm run build
```

## Windows Explorer Integration

The NSIS installer registers per-user Explorer context-menu entries for supported archive extensions. Right-clicking a supported archive shows GhostZip actions such as:

- Analyze archive with GhostZip
- Extract here with GhostZip
- Extract to archive folder with GhostZip

Explorer actions open GhostZip with the archive and output directory prefilled. The app analyzes space first and waits for extraction to be started from the UI.

## Current Limits

- "Zero-space" still requires enough free space for the current file plus the low-space floor.
- Only ZIP purge mode currently attempts to reclaim archive storage during extraction.
- The auto-delete setting is UI state only and defaults to on each app launch.
- A full end-to-end GUI extraction test is not currently automated in this repo.

## Deployment Notes

- App identifier: `com.ghostzip.app`
- Publisher: `GhostZip`
- Installer target: NSIS
- Expected installer name: `GhostZip_0.1.0_x64-setup.exe` unless customized by Tauri.

For SmartScreen trust, sign the installer and application binaries with an EV code-signing certificate, configure signing in the Tauri bundler or CI environment, timestamp signatures, and publish signed installers through a trusted release channel.
