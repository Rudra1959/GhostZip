import { spawn } from "node:child_process";
import { constants, mkdirSync, writeFileSync } from "node:fs";
import { access, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const mode = process.argv[2] ?? "dev";
const tempRoot = join(tmpdir(), "ghostzip-vite");
const cacheDir = process.env.GHOSTZIP_CACHE_DIR ?? join(tempRoot, "cache");
const outDir = process.env.GHOSTZIP_OUT_DIR ?? "dist";
const configPath = join(tempRoot, "vite.config.mjs");

mkdirSync(tempRoot, { recursive: true });

await warnIfBuildOutputIsNotWritable(mode, outDir);

writeFileSync(
  configPath,
  `export default {
  clearScreen: false,
  cacheDir: ${JSON.stringify(cacheDir.replaceAll("\\", "/"))},
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"]
    }
  },
  preview: {
    host: "127.0.0.1",
    port: 4173,
    strictPort: true
  },
  build: {
    outDir: ${JSON.stringify(outDir.replaceAll("\\", "/"))}
  }
};
`,
);

const viteBin = process.execPath;
const viteEntrypoint = join(repoRoot, "node_modules", "vite", "bin", "vite.js");

const viteArgs =
  mode === "build"
    ? [viteEntrypoint, "build", "--config", configPath]
    : mode === "preview"
      ? [viteEntrypoint, "preview", "--config", configPath]
      : [viteEntrypoint, "--config", configPath];

const child = spawn(viteBin, viteArgs, {
  cwd: repoRoot,
  env: process.env,
  stdio: "inherit",
  shell: false,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});

async function warnIfBuildOutputIsNotWritable(currentMode, configuredOutDir) {
  if (currentMode !== "build") return;

  const absoluteOutDir = resolve(repoRoot, configuredOutDir);
  try {
    mkdirSync(absoluteOutDir, { recursive: true });
    await access(absoluteOutDir, constants.W_OK);
    const probePath = join(absoluteOutDir, `.ghostzip-write-probe-${Date.now()}`);
    await writeFile(probePath, "ok");
    await unlink(probePath);
  } catch {
    console.warn(
      [
        "",
        "GhostZip build warning:",
        `Vite cannot write to ${absoluteOutDir}.`,
        "Move the repo to a normal writable folder such as C:\\dev\\Ghostzip,",
        "allow this folder in Windows Security, or set GHOSTZIP_OUT_DIR to a writable path for diagnostics.",
        "",
      ].join("\n"),
    );
  }
}
