# GhostZip Deployment Guide

This guide covers the normal Windows release path: build the Tauri app, publish the NSIS installer, and point the landing page download button at that installer.

## Recommended Local Path

Build from a normal writable development folder, for example:

```powershell
C:\dev\Ghostzip
```

Avoid protected Windows known folders if builds fail with `ENOENT` while creating temporary files or writing `dist`. If you keep the repo under `Videos`, `Documents`, or another protected folder, allow the folder in Windows Security Controlled Folder Access or move the repo before building.

For diagnostics only, you can redirect the web build output to a writable folder:

```powershell
$env:GHOSTZIP_OUT_DIR = "C:\tmp\ghostzip-dist"
npm run build
Remove-Item Env:\GHOSTZIP_OUT_DIR
```

Use the normal `dist` output for release builds because Tauri and the deployment steps expect that folder.

## Install Dependencies

```powershell
npm install
```

Required system tools:

- Node.js and npm
- Rust stable toolchain
- Microsoft Visual Studio Build Tools with the C++ desktop workload
- WebView2 Runtime

## Development

Run the browser landing page:

```powershell
npm run dev
```

Run the desktop extractor with Tauri:

```powershell
npm run tauri dev
```

## Production Build

Build the web assets only:

```powershell
npm run build
```

Build the Windows desktop installer:

```powershell
npm run tauri build
```

The NSIS installer is written to:

```text
src-tauri\target\release\bundle\nsis\GhostZip_0.1.0_x64-setup.exe
```

## Publish The Download

The landing page download button points to:

```text
/downloads/GhostZip_0.1.0_x64-setup.exe
```

After a successful Tauri build, copy the installer into your website host under:

```text
downloads\GhostZip_0.1.0_x64-setup.exe
```

For local static hosting from this repo, place that file under:

```text
public\downloads\GhostZip_0.1.0_x64-setup.exe
```

## Host The Landing Page

Build the landing page:

```powershell
npm run build
```

Upload the generated `dist` folder to any static web host. Examples:

- Vercel static deployment
- Netlify static deployment
- GitHub Pages
- Cloudflare Pages
- Any server that can serve static HTML, CSS, JS, and the installer file

## Code Signing

For real public distribution, sign the app and installer before publishing. Windows SmartScreen trust improves when you:

- Use a trusted code-signing certificate, preferably EV for faster reputation.
- Sign the Tauri executable and installer.
- Timestamp signatures.
- Publish signed installers from a stable domain.

## Release Checklist

- `npm run build` passes.
- `npm run tauri build` passes.
- The installer exists under `src-tauri\target\release\bundle\nsis`.
- The installer is copied to the hosted `downloads` path.
- The landing page download button returns the installer.
- A clean Windows machine can install GhostZip.
- Right-click archive actions appear after installation.
- ZIP Extract & Purge is tested on an NTFS or ReFS drive.
