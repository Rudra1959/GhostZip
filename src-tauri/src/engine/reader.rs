use std::path::Path;
use tauri::AppHandle;

use super::{ArchiveEntry, EngineResult, ExtractionSession, ExtractorEngine};

pub trait ArchiveReader: Send + Sync {
    fn extract_session(
        &self,
        app: AppHandle,
        engine: &ExtractorEngine,
        session: &ExtractionSession,
        helper_path: Option<&Path>,
    ) -> EngineResult<()>;
}
