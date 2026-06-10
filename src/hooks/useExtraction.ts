import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ArchiveManifest, ExtractionMode, LaunchContext, ProgressEvent, PurgeProgressEvent, Step } from '../types';

export function useExtraction() {
  const [step, setStep] = useState<Step>("drop");
  const [archivePath, setArchivePath] = useState("");
  const [outputDir, setOutputDir] = useState("");
  const [manifest, setManifest] = useState<ArchiveManifest | null>(null);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [paused, setPaused] = useState(false);
  const [archiveDeleted, setArchiveDeleted] = useState(false);
  
  // ETA calculation
  const [startTime, setStartTime] = useState<number | null>(null);
  const [eta, setEta] = useState<number | null>(null);

  // Purge progress tracking
  const [purgeProgress, setPurgeProgress] = useState<Map<string, boolean>>(new Map());
  const [archiveShrinkage, setArchiveShrinkage] = useState<number | null>(null);
  const [extractionMode, setExtractionMode] = useState<ExtractionMode>("normal");

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
    const unlistenComplete = listen<{ archiveDeleted?: boolean }>("extraction-complete", (event) => {
      setStep("complete");
      setBusy(false);
      if (event.payload && event.payload.archiveDeleted) {
        setArchiveDeleted(true);
      }
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

    // Listen for purge-progress events (Extract & Purge mode)
    const unlistenPurge = listen<PurgeProgressEvent>("purge-progress", (event) => {
      const { fileName, purgedFromArchive, archiveSizeAfter } = event.payload;
      setPurgeProgress(prev => {
        const next = new Map(prev);
        next.set(fileName, purgedFromArchive);
        return next;
      });
      setArchiveShrinkage(archiveSizeAfter);
    });

    return () => {
      void unlistenProgress.then((fn) => fn());
      void unlistenComplete.then((fn) => fn());
      void unlistenPause.then((fn) => fn());
      void unlistenError.then((fn) => fn());
      void unlistenPurge.then((fn) => fn());
    };
  }, []);

  const startExtraction = async (mode: ExtractionMode) => {
    if (!manifest) return;
    setBusy(true);
    setPaused(false);
    setError("");
    setArchiveDeleted(false);
    setStep("extracting");
    setStartTime(Date.now());
    setExtractionMode(mode);
    // Reset purge tracking when starting a new extraction
    setPurgeProgress(new Map());
    setArchiveShrinkage(mode === "extractAndPurge" ? manifest.compressedSize : null);
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
    manifest, setManifest, progress, eta,
    error, busy, paused, archiveDeleted,
    purgeProgress, archiveShrinkage, extractionMode,
    analyzeArchive, startExtraction, control, setError
  };
}
