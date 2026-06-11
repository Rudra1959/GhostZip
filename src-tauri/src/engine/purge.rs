use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

use super::archive::{safe_output_path, ArchiveFormat, EntryKind};
use super::constants::*;
use super::{
    EngineError, EngineResult, ExtractionMode, ExtractionSession, ExtractorEngine,
};

#[cfg(windows)]
use super::sparse;

const EVENT_PURGE_PROGRESS: &str = "purge-progress";
const EVENT_PROGRESS: &str = "extraction-progress";
const EVENT_COMPLETE: &str = "extraction-complete";
const EVENT_PAUSED: &str = "extraction-paused";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PurgeProgressEvent {
    file_name: String,
    purged_from_archive: bool,
    archive_size_after: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    file_name: String,
    file_index: usize,
    total_files: usize,
    bytes_extracted: u64,
    total_bytes: u64,
    free_space_remaining: u64,
}

/// Main entry point for Extract & Purge mode.
pub fn extract_and_purge(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    helper_path: Option<PathBuf>,
) -> EngineResult<()> {
    engine.set_mode(&session.manifest.session_id, ExtractionMode::Running)?;

    match session.manifest.format {
        ArchiveFormat::Zip => purge_zip(app, engine, session),
        ArchiveFormat::SevenZip | ArchiveFormat::Rar => {
            purge_via_7za(app, engine, session, helper_path)
        }
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz => purge_tar_fallback(app, engine, session),
    }
}

// ─── ZIP Extract & Purge via Sparse Files (Hole Punching) ───────────────────
//
// We use Windows Sparse Files (FSCTL_SET_SPARSE and FSCTL_SET_ZERO_DATA) to
// punch holes in the ZIP archive corresponding to the compressed data of each
// extracted file. This instantly frees disk space without rewriting the archive!
fn purge_zip(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
) -> EngineResult<()> {
    fs::create_dir_all(&session.manifest.output_dir).map_err(|source| EngineError::Io {
        context: format!("create {}", session.manifest.output_dir.display()),
        source,
    })?;

    let total_files = session.manifest.file_count;
    let mut extracted_bytes = 0_u64;
    let mut extracted_files = 0_usize;

    let file_entries: Vec<_> = session
        .manifest
        .entries
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .cloned()
        .collect();

    for entry in &session.manifest.entries {
        if entry.kind == EntryKind::Directory {
            let output = safe_output_path(&session.manifest.output_dir, &entry.path)?;
            let _ = fs::create_dir_all(&output);
        }
    }

    let archive_path = session.manifest.archive_path.clone();

    // Mark the archive as sparse.
    #[cfg(windows)]
    let archive_for_sparse = std::fs::OpenOptions::new()
        .write(true)
        .open(&archive_path)
        .ok();
    
    #[cfg(windows)]
    if let Some(ref f) = archive_for_sparse {
        let _ = sparse::mark_sparse(f);
    }

    for entry in &file_entries {
        check_control_state(app.clone(), engine, session)?;

        let disk = ExtractorEngine::disk_space_for(&session.manifest.output_dir)?;
        if disk.available_bytes < entry.uncompressed_size.saturating_add(LOW_SPACE_FLOOR_BYTES) {
            engine.set_mode(&session.manifest.session_id, ExtractionMode::PausedByLowSpace)?;
            let _ = app.emit(EVENT_PAUSED, serde_json::json!({ "reason": "low_space" }));
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                match engine.mode_for(&session.manifest.session_id)? {
                    ExtractionMode::Running => break,
                    ExtractionMode::Cancelled => {
                        return Err(EngineError::Archive("Extraction cancelled.".into()));
                    }
                    _ => continue,
                }
            }
        }

        let extraction_result: EngineResult<(u64, u64)> = (|| {
            let file = File::open(&archive_path).map_err(|source| EngineError::Io {
                context: format!("open archive {}", archive_path.display()),
                source,
            })?;
            let mut zip = ZipArchive::new(file)
                .map_err(|e| EngineError::Archive(format!("ZIP open failed: {e}")))?;

            let output_path = safe_output_path(&session.manifest.output_dir, &entry.path)?;

            if let Some(parent) = output_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut source = zip.by_name(&entry.path).map_err(|e| {
                EngineError::Archive(format!("ZIP entry {} not found: {e}", entry.path))
            })?;

            let data_start = source.data_start();
            let compressed_size = source.compressed_size();
            let expected_crc32 = source.crc32();

            let temp_output = output_path.with_extension("ghostzip-partial");
            let mut writer = File::create(&temp_output).map_err(|source| EngineError::Io {
                context: format!("create {}", output_path.display()),
                source,
            })?;

            let mut hasher = crc32fast::Hasher::new();
            let mut buffer = vec![0u8; BUFFER_SIZE];
            loop {
                let n = source.read(&mut buffer).map_err(|source| {
                    let _ = fs::remove_file(&temp_output);
                    EngineError::Io {
                        context: format!("read {}", entry.path),
                        source,
                    }
                })?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
                writer.write_all(&buffer[..n]).map_err(|source| {
                    let _ = fs::remove_file(&temp_output);
                    EngineError::Io {
                        context: format!("write {}", entry.path),
                        source,
                    }
                })?;
            }
            writer.flush().map_err(|source| EngineError::Io {
                context: format!("flush {}", entry.path),
                source,
            })?;
            drop(writer);
            drop(source);
            drop(zip);

            let metadata = fs::metadata(&temp_output).map_err(|source| EngineError::Io {
                context: format!("verify {}", entry.path),
                source,
            })?;
            if metadata.len() != entry.uncompressed_size && entry.uncompressed_size > 0 {
                let _ = fs::remove_file(&temp_output);
                return Err(EngineError::CorruptEntry {
                    path: entry.path.clone(),
                    detail: format!(
                        "Size mismatch: expected {} bytes, got {}",
                        entry.uncompressed_size,
                        metadata.len()
                    ),
                });
            }

            let computed_crc32 = hasher.finalize();
            if expected_crc32 != 0 && computed_crc32 != expected_crc32 {
                let _ = fs::remove_file(&temp_output);
                return Err(EngineError::CorruptEntry {
                    path: entry.path.clone(),
                    detail: format!(
                        "CRC32 mismatch: expected {:08X}, got {:08X}",
                        expected_crc32, computed_crc32
                    ),
                });
            }

            fs::rename(&temp_output, &output_path).map_err(|source| EngineError::Io {
                context: format!("finalize {}", output_path.display()),
                source,
            })?;

            Ok((data_start, compressed_size))
        })();

        let (data_start, compressed_size) = match extraction_result {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Failed to extract file {}: {:?}", entry.path, e);
                let _ = app.emit(
                    crate::engine::EVENT_ERROR,
                    serde_json::json!({
                        "fileName": entry.path.clone(),
                        "errorMessage": e.user_message()
                    }),
                );
                continue;
            }
        };

        extracted_bytes = extracted_bytes.saturating_add(entry.uncompressed_size);
        extracted_files += 1;

        let disk = ExtractorEngine::disk_space_for(&session.manifest.output_dir)?;
        let _ = app.emit(
            EVENT_PROGRESS,
            ProgressEvent {
                file_name: entry.path.clone(),
                file_index: extracted_files,
                total_files,
                bytes_extracted: extracted_bytes,
                total_bytes: session.manifest.total_uncompressed_size,
                free_space_remaining: disk.available_bytes,
            },
        );

        let mut purged = false;
        #[cfg(windows)]
        if let Some(ref f) = archive_for_sparse {
            if compressed_size > 0 {
                if let Ok(_) = sparse::punch_hole(f, data_start, compressed_size) {
                    purged = true;
                }
            } else {
                purged = true;
            }
        }

        let remaining_compressed = if session.manifest.total_uncompressed_size == 0 {
            0
        } else {
            session.manifest.compressed_size.saturating_sub(
                (extracted_bytes as f64 / session.manifest.total_uncompressed_size as f64
                    * session.manifest.compressed_size as f64) as u64
            )
        };

        let _ = app.emit(
            EVENT_PURGE_PROGRESS,
            PurgeProgressEvent {
                file_name: entry.path.clone(),
                purged_from_archive: purged,
                archive_size_after: remaining_compressed,
            },
        );
    }

    #[cfg(windows)]
    drop(archive_for_sparse);

    fs::remove_file(&archive_path).map_err(|source| EngineError::Io {
        context: format!("delete source archive {}", archive_path.display()),
        source,
    })?;

    engine.set_mode(&session.manifest.session_id, ExtractionMode::Complete)?;
    let _ = app.emit(
        EVENT_COMPLETE,
        serde_json::json!({
            "totalFiles": total_files,
            "outputDir": session.manifest.output_dir.display().to_string(),
            "archiveDeleted": true,
        }),
    );
    Ok(())
}

fn purge_via_7za(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    helper_path: Option<PathBuf>,
) -> EngineResult<()> {
    use super::extract;
    
    extract::extract_session(
        app.clone(),
        engine,
        session.clone(),
        super::UserExtractionMode::ExtractAndPurge,
        helper_path,
    )?;

    fs::remove_file(&session.manifest.archive_path).map_err(|source| EngineError::Io {
        context: format!("delete source archive {}", session.manifest.archive_path.display()),
        source,
    })?;

    let _ = app.emit(
        EVENT_COMPLETE,
        serde_json::json!({
            "totalFiles": session.manifest.file_count,
            "outputDir": session.manifest.output_dir.display().to_string(),
            "archiveDeleted": true,
        }),
    );

    Ok(())
}

fn purge_tar_fallback(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
) -> EngineResult<()> {
    use super::extract;
    extract::extract_session(
        app.clone(),
        engine,
        session.clone(),
        super::UserExtractionMode::ExtractAndPurge,
        None,
    )?;
    fs::remove_file(&session.manifest.archive_path).map_err(|source| EngineError::Io {
        context: format!("delete source archive {}", session.manifest.archive_path.display()),
        source,
    })?;

    let _ = app.emit(
        EVENT_COMPLETE,
        serde_json::json!({
            "totalFiles": session.manifest.file_count,
            "outputDir": session.manifest.output_dir.display().to_string(),
            "archiveDeleted": true,
        }),
    );
    Ok(())
}

fn check_control_state(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
) -> EngineResult<()> {
    loop {
        match engine.mode_for(&session.manifest.session_id)? {
            ExtractionMode::Running => return Ok(()),
            ExtractionMode::PausedByUser => {
                let _ = app.emit(EVENT_PAUSED, serde_json::json!({ "reason": "user_requested" }));
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            ExtractionMode::PausedByLowSpace => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            ExtractionMode::Cancelled => {
                return Err(EngineError::Archive("Extraction cancelled.".into()));
            }
            ExtractionMode::Complete => return Ok(()),
        }
    }
}
