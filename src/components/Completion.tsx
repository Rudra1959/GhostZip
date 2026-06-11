import React, { useState, useEffect } from 'react';
import { CheckCircle2, FolderOpen, Trash2, ArrowLeft } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function Completion({
  manifest, autoDelete, archiveDeleted, extractionMode, onOpenOutput, onReset
}: any) {
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [deleted, setDeleted] = useState(archiveDeleted || false);
  const [error, setError] = useState("");

  const handleDelete = async () => {
    if (deleted) return;
    try {
      await invoke("delete_archive", { path: manifest.archivePath });
      setDeleted(true);
      setDeleteConfirm(false);
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    if (archiveDeleted) {
      setDeleted(true);
    }
  }, [archiveDeleted]);

  useEffect(() => {
    if (autoDelete && !deleted && !archiveDeleted) {
      handleDelete();
    }
  }, [autoDelete, archiveDeleted]);

  return (
    <div className="completion-container">
      <div className="glass-panel" style={{ padding: '40px', maxWidth: '500px', width: '100%' }}>
        <CheckCircle2 size={80} className="success-icon pulse" />
        <h1 style={{ marginBottom: '16px' }}>Extraction Complete</h1>
        <p className="subtext">{manifest.fileCount} files successfully extracted to:</p>
        <div style={{ background: 'rgba(0,0,0,0.3)', padding: '12px', borderRadius: '8px', marginTop: '8px', marginBottom: '32px', fontFamily: 'monospace', fontSize: '13px', color: 'var(--accent-cyan)' }}>
          {manifest.outputDir}
        </div>

        {error && <div style={{ background: 'rgba(255,0,85,0.1)', color: 'var(--danger-color)', padding: '12px', borderRadius: '8px', marginBottom: '24px' }}>{error}</div>}

        <p className="subtext" style={{ marginBottom: '24px' }}>
          {deleted
            ? extractionMode === "extractAndPurge"
              ? "The source archive was removed after the purge extraction finished."
              : "The source archive has been deleted to recover disk space."
            : "The source archive is still available until you delete it."}
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <button className="primary" onClick={onOpenOutput} style={{ height: '48px', fontSize: '16px' }}>
            <FolderOpen size={20} /> Open Extracted Files
          </button>
          
          <div style={{ display: 'flex', gap: '16px' }}>
            <button onClick={onReset} style={{ flex: 1 }}>
              <ArrowLeft size={16} /> New Extraction
            </button>
            {!deleted ? (
              <button onClick={() => setDeleteConfirm(true)} className="purge-btn" style={{ flex: 1 }}>
                <Trash2 size={16} /> Delete Archive
              </button>
            ) : (
              <button disabled style={{ flex: 1, color: 'var(--success-color)' }}>
                <CheckCircle2 size={16} /> Archive Deleted
              </button>
            )}
          </div>
        </div>

        {deleteConfirm && !deleted && (
          <div className="glass-panel mt-4" style={{ padding: '20px', border: '1px solid var(--danger-color)', animation: 'pulse 2s infinite' }}>
            <strong style={{ color: 'var(--danger-color)', fontSize: '16px' }}>Delete source archive?</strong>
            <p className="subtext mt-2">This will permanently delete `{manifest.archivePath}`.</p>
            <div style={{ display: 'flex', gap: '12px', marginTop: '16px' }}>
              <button className="purge-btn" onClick={handleDelete} style={{ flex: 1 }}>Yes, Delete</button>
              <button onClick={() => setDeleteConfirm(false)} style={{ flex: 1 }}>Cancel</button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
