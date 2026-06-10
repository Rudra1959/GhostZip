import React from 'react';
import { Play, Pause, XCircle, HardDrive } from 'lucide-react';
import { formatBytes, formatEta } from '../utils';

export function Extraction({
  manifest, progress, eta, busy, paused, onPause, onResume, onCancel,
  purgeProgress, archiveShrinkage, extractionMode
}: any) {
  const completed = progress?.fileIndex ?? 0;
  const pct = progress ? Math.min(100, (progress.bytesExtracted / Math.max(progress.totalBytes, 1)) * 100) : 0;
  
  return (
    <div className="extraction-screen flex-center" style={{ width: '100%', height: '100%' }}>
      <div className="glass-panel" style={{ display: 'flex', flexDirection: 'column', width: '100%', maxWidth: '800px', padding: '24px' }}>
        
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '24px' }}>
          <HardDrive size={24} className="pulse" style={{ color: 'var(--accent-cyan)' }} />
          <div>
            <h2 style={{ margin: 0 }}>{paused ? "Extraction Paused" : "Extracting Archive..."}</h2>
            <div className="subtext truncate" style={{ maxWidth: '400px' }}>{manifest.archivePath}</div>
          </div>
        </div>

        <div className="progress-container">
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '14px', fontWeight: 500 }}>
            <span className="truncate" style={{ maxWidth: '70%' }}>{progress?.fileName ?? "Preparing files..."}</span>
            <span style={{ color: 'var(--accent-cyan)' }}>{Math.round(pct)}%</span>
          </div>
          <div className="progress-bar-bg">
            <div className="progress-bar-fill" style={{ width: `${pct}%` }} />
          </div>
        </div>

        <div className="extraction-stats">
          <div className="stat-box">
            <div className="stat-label">Extracted Data</div>
            <div className="stat-value">{formatBytes(progress?.bytesExtracted ?? 0)}</div>
            <div className="subtext mt-1">of {formatBytes(manifest.totalUncompressedSize)}</div>
          </div>
          <div className="stat-box">
            <div className="stat-label">Files Processed</div>
            <div className="stat-value purple">{completed}</div>
            <div className="subtext mt-1">of {manifest.fileCount} total files</div>
          </div>
          <div className="stat-box">
            <div className="stat-label">Time Remaining</div>
            <div className="stat-value" style={{ color: 'var(--text-primary)' }}>
              {paused ? "Paused" : eta ? formatEta(eta) : "Calculating..."}
            </div>
            <div className="subtext mt-1">Free Space: <span style={{ color: (progress?.freeSpaceRemaining ?? manifest.disk.availableBytes) < manifest.totalUncompressedSize ? 'var(--danger-color)' : 'var(--success-color)' }}>{formatBytes(progress?.freeSpaceRemaining ?? manifest.disk.availableBytes)}</span></div>
          </div>
          <div className="stat-box">
            <div className="stat-label">Archive Size</div>
            <div className="stat-value" style={{ color: extractionMode === "extractAndPurge" ? 'var(--success-color)' : 'var(--text-primary)' }}>
              {extractionMode === "extractAndPurge" && archiveShrinkage !== null ? formatBytes(archiveShrinkage) : formatBytes(manifest.compressedSize)}
            </div>
            <div className="subtext mt-1">Mode: {extractionMode === "extractAndPurge" ? "Zero-Space Purge" : "Standard"}</div>
          </div>
        </div>

        <div className="actions-bar" style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px', marginTop: '32px', paddingTop: '24px', borderTop: '1px solid var(--border-light)' }}>
          {paused ? (
            <button className="primary" onClick={onResume}><Play size={16} /> Resume</button>
          ) : (
            <button onClick={onPause}><Pause size={16} /> Pause</button>
          )}
          <button onClick={onCancel} style={{ color: 'var(--danger-color)', borderColor: 'var(--danger-color)' }}><XCircle size={16} /> Cancel</button>
        </div>
      </div>
    </div>
  );
}
