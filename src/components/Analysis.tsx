import React from 'react';
import { Play, Flame, Folder, File as FileIcon, HardDrive } from 'lucide-react';
import { formatBytes } from '../utils';
import { ExtractionMode } from '../types';

export function Analysis({ manifest, busy, onExtract }: {
  manifest: any;
  busy: boolean;
  onExtract: (mode: ExtractionMode) => void;
}) {
  return (
    <div className="analysis-screen flex-center" style={{ width: '100%', height: '100%' }}>
      <div className="glass-panel" style={{ display: 'flex', flexDirection: 'column', width: '100%', height: '100%', overflow: 'hidden' }}>
        
        {/* Toolbar */}
        <div className="toolbar">
          <button 
            className="primary" 
            disabled={busy || manifest.recommendedSpaceSaver}
            onClick={() => onExtract("normal")}
            title="Extract normally"
          >
            <Play size={18} />
            <span>Standard Extract</span>
          </button>

          <button 
            className="purge-btn pulse" 
            disabled={busy}
            onClick={() => onExtract("extractAndPurge")}
            title="Extract files one-by-one and delete them from the archive to save space"
            style={{ marginLeft: 'auto' }}
          >
            <Flame size={18} />
            <span>Zero-Space Extract (Purge)</span>
          </button>
        </div>

        {/* Address bar equivalent */}
        <div className="address-bar">
          <HardDrive size={16} />
          <span className="path-label">Archive:</span>
          <span className="path-value truncate">{manifest.archivePath}</span>
        </div>

        {/* File List */}
        <div className="file-list-container">
          <table className="file-table">
            <thead>
              <tr>
                <th className="col-name">Name</th>
                <th className="col-size">Size</th>
                <th className="col-packed">Packed</th>
                <th className="col-type">Type</th>
              </tr>
            </thead>
            <tbody>
              {manifest.entries.slice(0, 100).map((entry: any, i: number) => (
                <tr key={i}>
                  <td className="col-name truncate">
                    {entry.kind === 'directory' ? <Folder size={16} className="icon-dir"/> : <FileIcon size={16} className="icon-file"/>}
                    {entry.path.split('/').pop() || entry.path}
                  </td>
                  <td className="col-size">{entry.kind === 'directory' ? '--' : formatBytes(entry.uncompressedSize)}</td>
                  <td className="col-packed">{entry.kind === 'directory' ? '--' : formatBytes(entry.compressedSize)}</td>
                  <td className="col-type">{entry.kind === 'directory' ? 'Folder' : 'File'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Status bar */}
        <div className="status-bar">
          <span>{manifest.fileCount} files, {manifest.directoryCount} folders</span>
          <span>Total Size: <span style={{color: 'var(--accent-cyan)'}}>{formatBytes(manifest.totalUncompressedSize)}</span></span>
          <span>Free Space: <span style={{color: manifest.disk.availableBytes < manifest.totalUncompressedSize ? 'var(--danger-color)' : 'var(--success-color)'}}>{formatBytes(manifest.disk.availableBytes)}</span></span>
        </div>
      </div>
    </div>
  );
}
