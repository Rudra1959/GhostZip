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

export type PurgeProgressEvent = {
  fileName: string;
  purgedFromArchive: boolean;
  archiveSizeAfter: number;
};

export type ExtractionMode = "auto" | "normal" | "extractAndPurge";

export type Step = "drop" | "analysis" | "extracting" | "complete";

export type LaunchContext = {
  archivePath?: string;
  outputDir?: string;
  requestedAction: "open" | "analyze" | "extract-here" | "extract-to-folder" | string;
};
