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
