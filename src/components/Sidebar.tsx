import React from 'react';
import { Upload, ShieldCheck, HardDrive, Settings } from 'lucide-react';
import { GhostLogo } from './GhostLogo';

export function Sidebar({ step, manifest, onStepChange, onSettingsClick }: any) {
  return (
    <header className="sidebar glass-panel">
      <div className="brand">
        <GhostLogo size={24} />
        <div className="brand-mark">GhostZip</div>
      </div>
      <nav className="nav-menu">
        <button className={`nav-btn ${step === "drop" ? "active" : ""}`} onClick={() => onStepChange("drop")}>
          <Upload size={16} /> Open
        </button>
        <button className={`nav-btn ${step === "analysis" ? "active" : ""}`} disabled={!manifest} onClick={() => onStepChange("analysis")}>
          <ShieldCheck size={16} /> Analyze
        </button>
        <button className={`nav-btn ${step === "extracting" || step === "complete" ? "active" : ""}`} disabled={!manifest} onClick={() => onStepChange("extracting")}>
          <HardDrive size={16} /> Extract
        </button>
      </nav>
      <div className="sidebar-footer">
        <button className="icon-btn" onClick={onSettingsClick} title="Settings">
          <Settings size={18} />
        </button>
      </div>
    </header>
  );
}
