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
    manifest, setManifest, progress, eta, error, busy, paused, archiveDeleted,
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
            <Completion manifest={manifest} autoDelete={autoDelete} archiveDeleted={archiveDeleted}
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
