import React, { useState, useEffect } from 'react';
import { Archive, FileArchive, FolderOpen, ShieldCheck, UploadCloud } from 'lucide-react';
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
    <div className={`drop-screen flex-center ${isDragging ? 'dragging' : ''}`} style={{ height: '100%', justifyContent: 'center' }}>
      <div className="drop-zone glass-panel">
        {isDragging ? <UploadCloud size={64} className="drop-icon pulse" /> : <Archive size={64} className="drop-icon" />}
        <h1>{isDragging ? 'Drop it like it\'s hot' : 'Select Archive'}</h1>
        <p className="subtext" style={{ marginTop: '8px', marginBottom: '32px' }}>Next-generation extraction with zero-space purge technology.</p>
        
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', width: '100%' }}>
          <div>
            <label style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '1px' }}>Archive File</label>
            <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
              <input value={archivePath} onChange={(e) => onArchivePath(e.target.value)} placeholder="C:\path\to\archive.zip" readOnly />
              <button type="button" onClick={pickArchive} title="Browse archive"><FileArchive size={16} /></button>
            </div>
          </div>
          
          <div>
            <label style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '1px' }}>Destination</label>
            <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
              <input value={outputDir} onChange={(e) => onOutputDir(e.target.value)} placeholder="C:\path\to\extract" readOnly />
              <button type="button" onClick={pickOutput} title="Browse output folder"><FolderOpen size={16} /></button>
            </div>
          </div>
        </div>

        <button className="primary" disabled={busy || !archivePath} onClick={onAnalyze} style={{ marginTop: '32px', width: '100%', height: '48px', fontSize: '16px' }}>
          <ShieldCheck size={20} /> {busy ? "Analyzing Archive..." : "Analyze & Continue"}
        </button>
      </div>
    </div>
  );
}
