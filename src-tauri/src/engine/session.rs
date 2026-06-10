use serde::{Deserialize, Serialize};

use super::ArchiveManifest;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserExtractionMode {
    Auto,
    Normal,
    ExtractAndPurge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtractionMode {
    Running,
    PausedByUser,
    PausedByLowSpace,
    Cancelled,
    Complete,
}

#[derive(Clone, Debug)]
pub struct ExtractionSession {
    pub manifest: ArchiveManifest,
    pub mode: ExtractionMode,
    pub space_saver: bool,
}

impl ExtractionSession {
    pub fn new(manifest: ArchiveManifest, space_saver: bool) -> Self {
        Self {
            manifest,
            mode: ExtractionMode::Running,
            space_saver,
        }
    }
}
