mod archive;
mod constants;
mod errors;
mod extract;
mod purge;
mod scheduler;
mod session;
mod sevenzip_cli;
mod storage;

pub use constants::*;

pub use archive::{detect_archive_format, ArchiveEntry, ArchiveManifest, EntryKind};
pub use errors::{EngineError, EngineResult};
pub use scheduler::schedule_entries;
pub use session::{ExtractionMode, ExtractionSession, UserExtractionMode};
pub use storage::DiskSpaceInfo;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct ExtractorEngine {
    sessions: std::sync::Arc<Mutex<HashMap<String, ExtractionSession>>>,
}

impl ExtractorEngine {
    pub fn inspect_archive(
        &self,
        archive_path: PathBuf,
        output_dir: PathBuf,
        helper_path: Option<PathBuf>,
    ) -> EngineResult<ArchiveManifest> {
        let format = detect_archive_format(&archive_path)?;
        let mut entries = archive::inspect_entries(&archive_path, format, helper_path.as_deref())?;
        entries.retain(|entry| entry.kind.is_extractable());

        if entries.is_empty() {
            return Err(EngineError::EmptyArchive);
        }

        let total_uncompressed_size = entries.iter().map(|entry| entry.uncompressed_size).sum();
        let compressed_size = std::fs::metadata(&archive_path)
            .map_err(|source| EngineError::Io {
                context: format!("read archive metadata for {}", archive_path.display()),
                source,
            })?
            .len();
        let disk = Self::disk_space_for(&output_dir)?;
        let recommended_space_saver =
            should_use_space_saver(disk.available_bytes, total_uncompressed_size);
        let tight_space = !recommended_space_saver
            && disk.available_bytes < total_uncompressed_size.saturating_mul(2);
        let session_id = Uuid::new_v4().to_string();
        let ordered_entries = schedule_entries(&entries, recommended_space_saver);

        let manifest = ArchiveManifest {
            session_id: session_id.clone(),
            archive_path: archive_path.clone(),
            output_dir: output_dir.clone(),
            format,
            file_count: ordered_entries
                .iter()
                .filter(|entry| entry.kind == archive::EntryKind::File)
                .count(),
            directory_count: ordered_entries
                .iter()
                .filter(|entry| entry.kind == archive::EntryKind::Directory)
                .count(),
            compressed_size,
            total_uncompressed_size,
            disk,
            recommended_space_saver,
            tight_space,
            entries: ordered_entries,
        };

        self.sessions.lock().insert(
            session_id,
            ExtractionSession::new(manifest.clone(), recommended_space_saver),
        );
        Ok(manifest)
    }

    pub fn start_extraction(
        &self,
        app: AppHandle,
        session_id: String,
        selected_mode: UserExtractionMode,
        helper_path: Option<PathBuf>,
    ) -> EngineResult<()> {
        let session = self
            .sessions
            .lock()
            .get(&session_id)
            .cloned()
            .ok_or_else(|| EngineError::SessionNotFound(session_id.clone()))?;

        if selected_mode == UserExtractionMode::ExtractAndPurge {
            purge::extract_and_purge(app, self, &session, helper_path)
        } else {
            extract::extract_session(app, self, session, selected_mode, helper_path)
        }
    }

    pub fn set_mode(&self, session_id: &str, mode: ExtractionMode) -> EngineResult<()> {
        let mut sessions = self.sessions.lock();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| EngineError::SessionNotFound(session_id.to_string()))?;
        session.mode = mode;
        Ok(())
    }

    pub fn mode_for(&self, session_id: &str) -> EngineResult<ExtractionMode> {
        self.sessions
            .lock()
            .get(session_id)
            .map(|session| session.mode)
            .ok_or_else(|| EngineError::SessionNotFound(session_id.to_string()))
    }

    pub fn disk_space_for(path: impl AsRef<Path>) -> EngineResult<DiskSpaceInfo> {
        storage::disk_space_for(path.as_ref())
    }

    pub fn bundled_helper_path() -> PathBuf {
        sevenzip_cli::bundled_helper_from_workspace()
    }
}

pub fn should_use_space_saver(available_bytes: u64, total_uncompressed_size: u64) -> bool {
    available_bytes <= total_uncompressed_size
}

#[cfg(test)]
mod tests {
    use super::should_use_space_saver;

    #[test]
    fn triggers_space_saver_when_free_space_is_below_total_uncompressed_size() {
        let five_hundred_mb = 500 * 1024 * 1024;
        let two_gb = 2 * 1024 * 1024 * 1024;
        assert!(should_use_space_saver(five_hundred_mb, two_gb));
    }
}
#[cfg(windows)]
pub mod sparse;
