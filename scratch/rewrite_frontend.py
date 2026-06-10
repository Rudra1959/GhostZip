import os

os.makedirs("c:/Users/HP/Videos/Ghostzip/src/components", exist_ok=True)
os.makedirs("c:/Users/HP/Videos/Ghostzip/src/hooks", exist_ok=True)

types_ts = """
export type ArchiveFormat = "zip" | "rar" | "seven-zip" | "tar" | "tar-gz" | "tar-bz2" | "tar-xz";

export type DiskSpaceInfo = {
  mountPoint: string;
  totalBytes: number;
  availableBytes: number;
};

export type ArchiveEntry = {
  path: string;
  compressedSize: number;
  uncompressedSize: number;
  kind: "file" | "directory" | "symlink" | "other";
};

export type ArchiveManifest = {
  sessionId: string;
  archivePath: string;
  outputDir: string;
  format: ArchiveFormat;
  fileCount: number;
  directoryCount: number;
  compressedSize: number;
  totalUncompressedSize: number;
  disk: DiskSpaceInfo;
  recommendedSpaceSaver: boolean;
  tightSpace: boolean;
  entries: ArchiveEntry[];
};

export type ProgressEvent = {
  fileName: string;
  fileIndex: number;
  totalFiles: number;
  bytesExtracted: number;
  totalBytes: number;
  freeSpaceRemaining: number;
};

export type Step = "drop" | "analysis" | "extracting" | "complete";

export type LaunchContext = {
  archivePath?: string;
  outputDir?: string;
  requestedAction: "open" | "analyze" | "extract-here" | "extract-to-folder" | string;
};
"""

utils_ts = """
import { ArchiveFormat } from './types';

export const formatBytes = (value: number) => {
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  return `${(value / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
};

export const formatLabel: Record<ArchiveFormat, string> = {
  zip: "ZIP",
  rar: "RAR",
  "seven-zip": "7Z",
  tar: "TAR",
  "tar-gz": "TAR.GZ",
  "tar-bz2": "TAR.BZ2",
  "tar-xz": "TAR.XZ"
};

export const formatEta = (etaSeconds: number) => {
  if (!Number.isFinite(etaSeconds) || etaSeconds < 0) return "Calculating...";
  if (etaSeconds < 60) return `${Math.ceil(etaSeconds)}s`;
  const m = Math.floor(etaSeconds / 60);
  const s = Math.ceil(etaSeconds % 60);
  return `${m}m ${s}s`;
};
"""

hooks_useTheme_ts = """
import { useState, useEffect } from 'react';

export function useTheme() {
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("ghostzip-theme");
    return (saved as "dark" | "light") || "dark";
  });

  useEffect(() => {
    localStorage.setItem("ghostzip-theme", theme);
    document.documentElement.className = theme;
  }, [theme]);

  return [theme, setTheme] as const;
}
"""

hooks_useExtraction_ts = """
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ArchiveManifest, LaunchContext, ProgressEvent, Step } from '../types';

export function useExtraction() {
  const [step, setStep] = useState<Step>("drop");
  const [archivePath, setArchivePath] = useState("");
  const [outputDir, setOutputDir] = useState("");
  const [manifest, setManifest] = useState<ArchiveManifest | null>(null);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [paused, setPaused] = useState(false);
  
  // ETA calculation
  const [startTime, setStartTime] = useState<number | null>(null);
  const [eta, setEta] = useState<number | null>(null);

  const analyzeArchive = useCallback(async (path: string, output: string) => {
    setBusy(true);
    setError("");
    try {
      const resolvedOutput = output || await invoke<string>("default_output_dir_for_archive", { path });
      setArchivePath(path);
      setOutputDir(resolvedOutput);
      const result = await invoke<ArchiveManifest>("inspect_archive", { path, outputDir: resolvedOutput });
      setManifest(result);
      setStep("analysis");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const context = await invoke<LaunchContext>("get_launch_context");
        if (!context.archivePath) return;

        const output = context.outputDir ?? await invoke<string>("default_output_dir_for_archive", { path: context.archivePath });
        setArchivePath(context.archivePath);
        setOutputDir(output);
        setStep("drop");

        if (context.requestedAction !== "open") {
          setError(
            context.requestedAction === "extract-here"
              ? "Explorer request loaded: extract here. Review the analysis, then start extraction."
              : context.requestedAction === "extract-to-folder"
                ? "Explorer request loaded: extract to archive folder. Review the analysis, then start extraction."
                : "Explorer request loaded. Review the analysis, then start extraction."
          );
          await analyzeArchive(context.archivePath, output);
        }
      } catch (err) {
        setError(String(err));
      }
    })();
  }, [analyzeArchive]);

  useEffect(() => {
    const unlistenProgress = listen<ProgressEvent>("extraction-progress", (event) => {
      setProgress(event.payload);
      setStep("extracting");
      
      setStartTime(prev => {
        if (!prev) return Date.now();
        const elapsed = (Date.now() - prev) / 1000;
        const speed = event.payload.bytesExtracted / Math.max(elapsed, 1);
        const remaining = event.payload.totalBytes - event.payload.bytesExtracted;
        setEta(remaining / Math.max(speed, 1));
        return prev;
      });
    });
    const unlistenComplete = listen("extraction-complete", () => {
      setStep("complete");
      setBusy(false);
    });
    const unlistenPause = listen<{ reason: string }>("extraction-paused", (event) => {
      setPaused(true);
      setBusy(false);
      setError(event.payload.reason === "low_space" ? "Extraction paused because disk space is critically low." : "Extraction paused.");
      setStartTime(null);
    });
    const unlistenError = listen<{ fileName: string; errorMessage: string }>("extraction-error", (event) => {
      setBusy(false);
      setError(`${event.payload.fileName}: ${event.payload.errorMessage}`);
    });

    return () => {
      void unlistenProgress.then((fn) => fn());
      void unlistenComplete.then((fn) => fn());
      void unlistenPause.then((fn) => fn());
      void unlistenError.then((fn) => fn());
    };
  }, []);

  const startExtraction = async (mode: "normal" | "spaceSaver" | "auto") => {
    if (!manifest) return;
    setBusy(true);
    setPaused(false);
    setError("");
    setStep("extracting");
    setStartTime(Date.now());
    try {
      await invoke("start_extraction", { sessionId: manifest.sessionId, mode });
    } catch (err) {
      setBusy(false);
      setError(String(err));
    }
  };

  const control = async (command: "pause_extraction" | "resume_extraction" | "cancel_extraction") => {
    if (!manifest) return;
    setError("");
    try {
      await invoke(command, { sessionId: manifest.sessionId });
      if (command === "pause_extraction") {
        setPaused(true);
        setStartTime(null);
      }
      if (command === "resume_extraction") {
        setPaused(false);
        setStartTime(Date.now());
      }
      if (command === "cancel_extraction") {
        setBusy(false);
        setPaused(false);
        setStep("analysis");
        setStartTime(null);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  return {
    step, setStep,
    archivePath, setArchivePath,
    outputDir, setOutputDir,
    manifest, progress, eta,
    error, busy, paused,
    analyzeArchive, startExtraction, control, setError
  };
}
"""

components_DropZone_tsx = """
import React, { useState, useEffect } from 'react';
import { Archive, FileArchive, FolderOpen, ShieldCheck, HardDrive, UploadCloud } from 'lucide-react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function DropZone({
  archivePath, outputDir, busy, onArchivePath, onOutputDir, onAnalyze
}: any) {
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const unlistenFileDrop = listen<{ paths: string[] }>('tauri://file-drop', (event) => {
      setIsDragging(false);
      if (event.payload.paths.length > 0) {
        handleFileSelect(event.payload.paths[0]);
      }
    });
    
    const unlistenFileHover = listen('tauri://file-drop-hover', () => setIsDragging(true));
    const unlistenFileCancel = listen('tauri://file-drop-cancelled', () => setIsDragging(false));

    return () => {
      unlistenFileDrop.then(fn => fn());
      unlistenFileHover.then(fn => fn());
      unlistenFileCancel.then(fn => fn());
    };
  }, []);

  const handleFileSelect = async (path: string) => {
    onArchivePath(path);
    if (!outputDir) {
      const defaultOutput = await invoke<string>("default_output_dir_for_archive", { path });
      onOutputDir(defaultOutput);
    }
  };

  const pickArchive = async () => {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: "Archives", extensions: ["zip", "7z", "tar", "gz", "tgz", "bz2", "xz", "rar"] }]
    });
    if (typeof selected === "string") handleFileSelect(selected);
  };

  const pickOutput = async () => {
    const selected = await openDialog({ multiple: false, directory: true });
    if (typeof selected === "string") onOutputDir(selected);
  };

  return (
    <div className={`drop-screen ${isDragging ? 'dragging' : ''}`}>
      <div className="drop-zone glass-card">
        {isDragging ? <UploadCloud size={64} className="pulse-icon" /> : <Archive size={58} className="float-icon" />}
        <h2>{isDragging ? 'Drop archive here' : 'Choose an archive to extract'}</h2>
        <p>ZIP, 7Z, TAR, RAR extraction with intelligent space management.</p>
        
        <div className="input-group">
          <label>Archive path</label>
          <div className="field-row">
            <input value={archivePath} onChange={(e) => onArchivePath(e.target.value)} placeholder="C:\\archive.zip" readOnly />
            <button type="button" onClick={pickArchive} title="Choose archive"><FileArchive size={18} /> Browse</button>
          </div>
        </div>
        
        <div className="input-group">
          <label>Output directory</label>
          <div className="field-row">
            <input value={outputDir} onChange={(e) => onOutputDir(e.target.value)} placeholder="C:\\Extracted" readOnly />
            <button type="button" onClick={pickOutput} title="Choose output folder"><FolderOpen size={18} /> Browse</button>
          </div>
        </div>

        <button className="primary mt-4" disabled={busy || !archivePath} onClick={onAnalyze}>
          <ShieldCheck size={18} /> {busy ? "Analyzing..." : "Analyze Archive"}
        </button>
      </div>
    </div>
  );
}
"""

components_Analysis_tsx = """
import React from 'react';
import { Play, HardDrive } from 'lucide-react';
import { formatBytes, formatLabel } from '../utils';

export function Analysis({ manifest, busy, onExtract }: any) {
  const status = manifest.recommendedSpaceSaver ? "critical" : manifest.tightSpace ? "tight" : "ready";
  const pct = Math.min(100, (manifest.totalUncompressedSize / Math.max(manifest.disk.availableBytes, 1)) * 100);
  
  return (
    <div className="analysis-screen fade-in">
      <header className="screen-header">
        <div>
          <h2>Archive analysis</h2>
          <p className="truncate">{manifest.archivePath}</p>
        </div>
        <div className={`status-badge ${status}`}>
          {status === "ready" ? "Enough space" : status === "tight" ? "Space-Saver recommended" : "Space-Saver required"}
        </div>
      </header>

      <div className="metric-grid">
        <Metric label="Format" value={formatLabel[manifest.format]} />
        <Metric label="Files" value={`${manifest.fileCount}`} />
        <Metric label="Compressed" value={formatBytes(manifest.compressedSize)} />
        <Metric label="Uncompressed" value={formatBytes(manifest.totalUncompressedSize)} />
        <Metric label="Free space" value={formatBytes(manifest.disk.availableBytes)} />
        <Metric label="Target drive" value={manifest.disk.mountPoint} />
      </div>

      <div className="glass-card mt-4">
        <h3>Space-Saver plan</h3>
        <p className="subtext">Files are extracted sequentially, prioritizing directories and smaller files when space is constrained.</p>
        <div className="gauge-container mt-3">
          <div className="gauge-track">
            <div className={`gauge-fill ${status}`} style={{ width: `${pct}%` }} />
          </div>
          <div className="gauge-labels">
            <span>0%</span>
            <span>Uncompressed / Free Space</span>
            <span>100%</span>
          </div>
        </div>
      </div>

      <div className="table-container mt-4">
        <div className="table-head">
          <span>Path</span><span>Size</span>
        </div>
        <div className="table-body">
          {manifest.entries.slice(0, 50).map((entry: any, i: number) => (
            <div className="table-row" key={i}>
              <span className="truncate" title={entry.path}>{entry.path}</span>
              <strong>{formatBytes(entry.uncompressedSize)}</strong>
            </div>
          ))}
        </div>
      </div>

      <div className="actions-bar mt-4">
        <button disabled={busy || manifest.recommendedSpaceSaver} onClick={() => onExtract("normal")}>
          <Play size={18} /> Extract Normal
        </button>
        <button className="primary" disabled={busy} onClick={() => onExtract("spaceSaver")}>
          <HardDrive size={18} /> Extract Space-Saver
        </button>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric glass-card">
      <span className="metric-label">{label}</span>
      <strong className="metric-value">{value}</strong>
    </div>
  );
}
"""

components_Extraction_tsx = """
import React from 'react';
import { Play, Pause, XCircle, CheckCircle2, Archive } from 'lucide-react';
import { formatBytes, formatEta } from '../utils';

export function Extraction({
  manifest, progress, eta, busy, paused, onPause, onResume, onCancel
}: any) {
  const completed = progress?.fileIndex ?? 0;
  const pct = progress ? Math.min(100, (progress.bytesExtracted / Math.max(progress.totalBytes, 1)) * 100) : 0;
  const visibleFiles = manifest.entries.filter((e: any) => e.kind === "file").slice(0, 100);

  return (
    <div className="extract-screen fade-in">
      <header className="screen-header">
        <div>
          <h2>{paused ? "Extraction paused" : "Extracting archive"}</h2>
          <p className="truncate current-file">{progress?.fileName ?? "Preparing files..."}</p>
        </div>
        <div className="status-badge ready">{completed} / {manifest.fileCount}</div>
      </header>

      <div className="glass-card progress-card mt-4">
        <div className="progress-bar-container">
          <div className="progress-bar-fill shimmer" style={{ width: `${pct}%` }} />
        </div>
        <div className="progress-stats mt-2">
          <span>{formatBytes(progress?.bytesExtracted ?? 0)} of {formatBytes(manifest.totalUncompressedSize)}</span>
          <span className="pct-text">{Math.round(pct)}%</span>
        </div>
      </div>

      <div className="metric-grid compact mt-4">
        <Metric label="Free space" value={formatBytes(progress?.freeSpaceRemaining ?? manifest.disk.availableBytes)} />
        <Metric label="ETA" value={paused ? "Paused" : eta ? formatEta(eta) : "Calculating..."} />
        <Metric label="Mode" value={manifest.recommendedSpaceSaver ? "Space-Saver" : "Normal"} />
      </div>

      <div className="table-container mt-4 extraction-list">
        {visibleFiles.map((file: any, index: number) => (
          <div className={`table-row ${index === completed ? 'active-row' : ''}`} key={file.path}>
            <div className="row-icon">
              {index < completed ? <CheckCircle2 className="text-green" size={16} /> : 
               index === completed ? <Archive className="text-accent pulse-icon" size={16} /> : 
               <div className="dot" />}
            </div>
            <span className="truncate" title={file.path}>{file.path}</span>
            <strong>{formatBytes(file.uncompressedSize)}</strong>
          </div>
        ))}
      </div>

      <div className="actions-bar mt-4">
        {paused ? (
          <button className="primary" onClick={onResume}><Play size={18} /> Resume</button>
        ) : (
          <button onClick={onPause}><Pause size={18} /> Pause</button>
        )}
        <button className="danger" onClick={onCancel}><XCircle size={18} /> Cancel</button>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric glass-card compact">
      <span className="metric-label">{label}</span>
      <strong className="metric-value">{value}</strong>
    </div>
  );
}
"""

components_Completion_tsx = """
import React, { useState } from 'react';
import { CheckCircle2, FolderOpen, Trash2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function Completion({
  manifest, autoDelete, onOpenOutput, onReset
}: any) {
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [deleted, setDeleted] = useState(false);
  const [error, setError] = useState("");

  const handleDelete = async () => {
    try {
      await invoke("delete_archive", { path: manifest.archivePath });
      setDeleted(true);
      setDeleteConfirm(false);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="complete-screen fade-in flex-center">
      <div className="success-icon scale-in">
        <CheckCircle2 size={80} />
      </div>
      <h2>Extraction Complete</h2>
      <p className="subtext mt-2">{manifest.fileCount} files extracted successfully.</p>
      <p className="path-text mt-1 truncate max-w-md">{manifest.outputDir}</p>

      {error && <div className="error-banner mt-4">{error}</div>}

      <div className="actions-bar mt-6">
        <button className="primary" onClick={onOpenOutput}><FolderOpen size={18} /> Open Folder</button>
        {!deleted ? (
          <button className="danger outline" onClick={() => setDeleteConfirm(true)}><Trash2 size={18} /> Delete Archive</button>
        ) : (
          <button className="success" disabled><CheckCircle2 size={18} /> Archive Deleted</button>
        )}
        <button onClick={onReset}>Extract Another</button>
      </div>

      {deleteConfirm && !deleted && (
        <div className="glass-card danger-card mt-4 slide-up">
          <strong>Delete source archive?</strong>
          <p className="text-sm mt-1">This will permanently delete `{manifest.archivePath}`.</p>
          <div className="actions-bar mt-3">
            <button className="danger" onClick={handleDelete}>Yes, delete it</button>
            <button onClick={() => setDeleteConfirm(false)}>Cancel</button>
          </div>
        </div>
      )}
    </div>
  );
}
"""

components_Sidebar_tsx = """
import React from 'react';
import { Upload, ShieldCheck, HardDrive, Settings } from 'lucide-react';

export function Sidebar({ step, manifest, onStepChange, onSettingsClick }: any) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">GZ</div>
        <div>
          <h1>GhostZip</h1>
          <p>Space-aware extractor</p>
        </div>
      </div>
      <nav className="nav-menu">
        <button className={`nav-btn ${step === "drop" ? "active" : ""}`} onClick={() => onStepChange("drop")}>
          <Upload size={18} /> Drop
        </button>
        <button className={`nav-btn ${step === "analysis" ? "active" : ""}`} disabled={!manifest} onClick={() => onStepChange("analysis")}>
          <ShieldCheck size={18} /> Analyze
        </button>
        <button className={`nav-btn ${step === "extracting" || step === "complete" ? "active" : ""}`} disabled={!manifest} onClick={() => onStepChange("extracting")}>
          <HardDrive size={18} /> Extract
        </button>
      </nav>
      <div className="sidebar-footer">
        <button className="nav-btn settings-btn" onClick={onSettingsClick}>
          <Settings size={18} /> Settings
        </button>
      </div>
    </aside>
  );
}
"""

components_SettingsPanel_tsx = """
import React from 'react';
import { X } from 'lucide-react';

export function SettingsPanel({
  theme, autoDelete, notifications, onTheme, onAutoDelete, onNotifications, onClose
}: any) {
  return (
    <div className="drawer-overlay fade-in">
      <div className="drawer-panel slide-in-right">
        <header className="drawer-header">
          <h2>Settings</h2>
          <button className="icon-btn" onClick={onClose}><X size={20} /></button>
        </header>
        
        <div className="settings-content">
          <div className="setting-group">
            <label>Theme</label>
            <select className="glass-select" value={theme} onChange={(e) => onTheme(e.target.value)}>
              <option value="dark">Dark Theme</option>
              <option value="light">Light Theme</option>
            </select>
          </div>
          
          <div className="setting-group checkbox-group">
            <label className="checkbox-label">
              <input type="checkbox" checked={autoDelete} onChange={(e) => onAutoDelete(e.target.checked)} />
              <span className="checkbox-text">
                <strong>Ask to delete archive</strong>
                <span>Prompt to delete the source archive after extraction completes</span>
              </span>
            </label>
          </div>

          <div className="setting-group checkbox-group">
            <label className="checkbox-label">
              <input type="checkbox" checked={notifications} onChange={(e) => onNotifications(e.target.checked)} />
              <span className="checkbox-text">
                <strong>System notifications</strong>
                <span>Show desktop notifications when extraction completes</span>
              </span>
            </label>
          </div>
        </div>
      </div>
    </div>
  );
}
"""

app_tsx = """
import React, { useState } from 'react';
import { openPath } from '@tauri-apps/plugin-opener';
import { useExtraction } from './hooks/useExtraction';
import { useTheme } from './hooks/useTheme';
import { Sidebar } from './components/Sidebar';
import { DropZone } from './components/DropZone';
import { Analysis } from './components/Analysis';
import { Extraction } from './components/Extraction';
import { Completion } from './components/Completion';
import { SettingsPanel } from './components/SettingsPanel';
import './styles.css';

export default function App() {
  const {
    step, setStep, archivePath, setArchivePath, outputDir, setOutputDir,
    manifest, progress, eta, error, busy, paused,
    analyzeArchive, startExtraction, control, setError
  } = useExtraction();

  const [theme, setTheme] = useTheme();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [autoDelete, setAutoDelete] = useState(true);
  const [notifications, setNotifications] = useState(true);

  return (
    <div className={`app-container ${theme}`}>
      <Sidebar step={step} manifest={manifest} onStepChange={setStep} onSettingsClick={() => setSettingsOpen(true)} />
      
      <main className="main-content">
        {error && (
          <div className="error-banner">
            <span>{error}</span>
            <button className="close-btn" onClick={() => setError("")}>×</button>
          </div>
        )}
        
        <div className="view-container">
          {step === "drop" && (
            <DropZone archivePath={archivePath} outputDir={outputDir} busy={busy}
              onArchivePath={setArchivePath} onOutputDir={setOutputDir} onAnalyze={() => analyzeArchive(archivePath, outputDir)} />
          )}
          {step === "analysis" && manifest && (
            <Analysis manifest={manifest} busy={busy} onExtract={startExtraction} />
          )}
          {step === "extracting" && manifest && (
            <Extraction manifest={manifest} progress={progress} eta={eta} busy={busy} paused={paused}
              onPause={() => control("pause_extraction")} onResume={() => control("resume_extraction")} onCancel={() => control("cancel_extraction")} />
          )}
          {step === "complete" && manifest && (
            <Completion manifest={manifest} autoDelete={autoDelete} 
              onOpenOutput={() => openPath(manifest.outputDir)} 
              onReset={() => { setArchivePath(""); setOutputDir(""); setStep("drop"); setManifest(null); }} />
          )}
        </div>
      </main>

      {settingsOpen && (
        <SettingsPanel theme={theme} autoDelete={autoDelete} notifications={notifications}
          onTheme={setTheme} onAutoDelete={setAutoDelete} onNotifications={setNotifications} onClose={() => setSettingsOpen(false)} />
      )}
    </div>
  );
}
"""

main_tsx = """
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"""

files = {
  "c:/Users/HP/Videos/Ghostzip/src/types.ts": types_ts,
  "c:/Users/HP/Videos/Ghostzip/src/utils.ts": utils_ts,
  "c:/Users/HP/Videos/Ghostzip/src/hooks/useTheme.ts": hooks_useTheme_ts,
  "c:/Users/HP/Videos/Ghostzip/src/hooks/useExtraction.ts": hooks_useExtraction_ts,
  "c:/Users/HP/Videos/Ghostzip/src/components/DropZone.tsx": components_DropZone_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/components/Analysis.tsx": components_Analysis_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/components/Extraction.tsx": components_Extraction_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/components/Completion.tsx": components_Completion_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/components/Sidebar.tsx": components_Sidebar_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/components/SettingsPanel.tsx": components_SettingsPanel_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/App.tsx": app_tsx,
  "c:/Users/HP/Videos/Ghostzip/src/main.tsx": main_tsx
}

for path, content in files.items():
    with open(path, "w", encoding="utf-8") as f:
        f.write(content.strip() + "\\n")

print("Frontend files rewritten successfully!")
