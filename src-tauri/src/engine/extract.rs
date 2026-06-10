use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use serde::Serialize;
use sevenz_rust::{decompress_file_with_extract_fn, Error as SevenZError};
use tar::Archive as TarArchive;
use tauri::{AppHandle, Emitter};
use xz2::read::XzDecoder;
use zip::ZipArchive;

use super::archive::{safe_output_path, ArchiveFormat, EntryKind};
use super::{
    sevenzip_cli, EngineError, EngineResult, ExtractionMode, ExtractionSession, ExtractorEngine,
    UserExtractionMode,
};
use super::constants::*;

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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompleteEvent {
    total_files: usize,
    output_dir: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PauseEvent {
    reason: &'static str,
}

pub fn extract_session(
    app: AppHandle,
    engine: &ExtractorEngine,
    mut session: ExtractionSession,
    selected_mode: UserExtractionMode,
    helper_path: Option<PathBuf>,
) -> EngineResult<()> {
    session.space_saver = match selected_mode {
        UserExtractionMode::Auto => session.space_saver,
        UserExtractionMode::Normal => false,
        UserExtractionMode::ExtractAndPurge => true,
    };
    engine.set_mode(&session.manifest.session_id, ExtractionMode::Running)?;

    match session.manifest.format {
        ArchiveFormat::Zip => extract_zip(app, engine, &session),
        ArchiveFormat::Tar => extract_tar(
            app,
            engine,
            &session,
            File::open(&session.manifest.archive_path).map_err(|source| EngineError::Io {
                context: format!("open {}", session.manifest.archive_path.display()),
                source,
            })?,
        ),
        ArchiveFormat::TarGz => extract_tar(
            app,
            engine,
            &session,
            GzDecoder::new(
                File::open(&session.manifest.archive_path).map_err(|source| EngineError::Io {
                    context: format!("open {}", session.manifest.archive_path.display()),
                    source,
                })?,
            ),
        ),
        ArchiveFormat::TarBz2 => extract_tar(
            app,
            engine,
            &session,
            bzip2::read::BzDecoder::new(File::open(&session.manifest.archive_path).map_err(
                |source| EngineError::Io {
                    context: format!("open {}", session.manifest.archive_path.display()),
                    source,
                },
            )?),
        ),
        ArchiveFormat::TarXz => extract_tar(
            app,
            engine,
            &session,
            XzDecoder::new(
                File::open(&session.manifest.archive_path).map_err(|source| EngineError::Io {
                    context: format!("open {}", session.manifest.archive_path.display()),
                    source,
                })?,
            ),
        ),
        ArchiveFormat::SevenZip => extract_7z(app, engine, &session),
        ArchiveFormat::Rar => extract_rar(app, engine, &session, helper_path),
    }
}

struct DiskThrottle {
    bytes_since_last_check: u64,
    files_since_last_check: usize,
    last_known_free: u64,
}

impl DiskThrottle {
    fn new() -> Self {
        Self {
            bytes_since_last_check: DISK_CHECK_INTERVAL_BYTES,
            files_since_last_check: DISK_CHECK_INTERVAL_FILES,
            last_known_free: 0,
        }
    }
}

fn ensure_space_or_pause(
    app: &AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    throttle: &mut DiskThrottle,
    output_dir: &Path,
    needed_bytes: u64,
) -> EngineResult<()> {
    loop {
        throttle.bytes_since_last_check = throttle.bytes_since_last_check.saturating_add(needed_bytes);
        throttle.files_since_last_check += 1;

        if throttle.bytes_since_last_check >= DISK_CHECK_INTERVAL_BYTES || throttle.files_since_last_check >= DISK_CHECK_INTERVAL_FILES {
            let disk = ExtractorEngine::disk_space_for(output_dir)?;
            throttle.last_known_free = disk.available_bytes;
            throttle.bytes_since_last_check = 0;
            throttle.files_since_last_check = 0;
        }

        if throttle.last_known_free < needed_bytes.saturating_add(LOW_SPACE_FLOOR_BYTES) {
            let _ = engine.set_mode(&session.manifest.session_id, ExtractionMode::PausedByLowSpace);
            let _ = app.emit(
                EVENT_PAUSED,
                PauseEvent {
                    reason: "low_space",
                },
            );
            std::thread::sleep(std::time::Duration::from_millis(1000));
            check_control_state(app.clone(), engine, session)?;
            throttle.bytes_since_last_check = DISK_CHECK_INTERVAL_BYTES;
            continue;
        }
        
        throttle.last_known_free = throttle.last_known_free.saturating_sub(needed_bytes);
        return Ok(());
    }
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
                let _ = app.emit(
                    EVENT_PAUSED,
                    PauseEvent {
                        reason: "user_requested",
                    },
                );
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            ExtractionMode::PausedByLowSpace => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            ExtractionMode::Cancelled => return Err(EngineError::Archive("Extraction cancelled.".to_string())),
            ExtractionMode::Complete => return Ok(()),
        }
    }
}

fn emit_progress(
    app: &AppHandle,
    session: &ExtractionSession,
    throttle: &mut DiskThrottle,
    file_name: String,
    file_index: usize,
    total_files: usize,
    bytes_extracted: u64,
) {
    let _ = app.emit(
        EVENT_PROGRESS,
        ProgressEvent {
            file_name,
            file_index,
            total_files,
            bytes_extracted,
            total_bytes: session.manifest.total_uncompressed_size,
            free_space_remaining: throttle.last_known_free,
        },
    );
}

fn complete(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    total_files: usize,
) -> EngineResult<()> {
    engine.set_mode(&session.manifest.session_id, ExtractionMode::Complete)?;
    let _ = app.emit(
        EVENT_COMPLETE,
        CompleteEvent {
            total_files,
            output_dir: session.manifest.output_dir.display().to_string(),
        },
    );
    Ok(())
}

fn extract_zip(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
) -> EngineResult<()> {
    fs::create_dir_all(&session.manifest.output_dir).map_err(|source| EngineError::Io {
        context: format!("create {}", session.manifest.output_dir.display()),
        source,
    })?;

    let file = File::open(&session.manifest.archive_path).map_err(|source| EngineError::Io {
        context: format!("open {}", session.manifest.archive_path.display()),
        source,
    })?;
    let mut archive = ZipArchive::new(file)
        .map_err(|source| EngineError::Archive(format!("ZIP extraction failed: {source}")))?;
    let total_files = session.manifest.file_count;
    let mut extracted_bytes = 0_u64;
    let mut extracted_files = 0_usize;
    let mut throttle = DiskThrottle::new();

    for manifest_entry in &session.manifest.entries {
        check_control_state(app.clone(), engine, session)?;

        if manifest_entry.kind == EntryKind::Directory {
            let output = safe_output_path(&session.manifest.output_dir, &manifest_entry.path)?;
            if let Err(e) = fs::create_dir_all(&output) {
                eprintln!("Failed to create directory {}: {}", manifest_entry.path, e);
            }
            continue;
        }

        let mut source = match archive.by_name(&manifest_entry.path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to open ZIP entry {}: {}", manifest_entry.path, e);
                continue;
            }
        };

        let output = safe_output_path(&session.manifest.output_dir, &manifest_entry.path)?;
        ensure_space_or_pause(&app, engine, session, &mut throttle, &session.manifest.output_dir, manifest_entry.uncompressed_size)?;

        if let Err(e) = copy_entry(
            app.clone(),
            engine,
            session,
            &mut source,
            &output,
            manifest_entry.path.as_str(),
        ) {
            eprintln!("Failed to extract file {}: {:?}", manifest_entry.path, e);
        } else {
            extracted_bytes = extracted_bytes.saturating_add(manifest_entry.uncompressed_size);
            extracted_files += 1;
            emit_progress(
                &app,
                session,
                &mut throttle,
                manifest_entry.path.clone(),
                extracted_files,
                total_files,
                extracted_bytes,
            );
        }
    }

    complete(app, engine, session, total_files)
}

fn extract_tar<R: Read>(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    reader: R,
) -> EngineResult<()> {
    fs::create_dir_all(&session.manifest.output_dir).map_err(|source| EngineError::Io {
        context: format!("create {}", session.manifest.output_dir.display()),
        source,
    })?;
    let mut archive = TarArchive::new(reader);
    let total_files = session.manifest.file_count;
    let mut extracted_bytes = 0_u64;
    let mut extracted_files = 0_usize;
    let mut throttle = DiskThrottle::new();

    let entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => return Err(EngineError::Archive(format!("TAR extraction failed: {e}"))),
    };

    for entry in entries {
        check_control_state(app.clone(), engine, session)?;

        let mut entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("TAR entry failed: {}", e);
                continue;
            }
        };

        let path = match entry.path() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => {
                eprintln!("TAR path read failed: {}", e);
                continue;
            }
        };

        let header_size = entry.header().size().unwrap_or(0);
        let entry_type = entry.header().entry_type();

        if entry_type == tar::EntryType::Symlink {
            continue;
        }

        let output = match safe_output_path(&session.manifest.output_dir, &path) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if entry_type == tar::EntryType::Directory {
            let _ = fs::create_dir_all(output);
            continue;
        }

        ensure_space_or_pause(&app, engine, session, &mut throttle, &session.manifest.output_dir, header_size)?;

        if let Err(e) = copy_entry(app.clone(), engine, session, &mut entry, &output, &path) {
            eprintln!("Failed to copy TAR entry {}: {:?}", path, e);
        } else {
            extracted_bytes = extracted_bytes.saturating_add(header_size);
            extracted_files += 1;
            emit_progress(
                &app,
                session,
                &mut throttle,
                path,
                extracted_files,
                total_files,
                extracted_bytes,
            );
        }
    }

    complete(app, engine, session, total_files)
}

fn extract_7z(
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
    let mut throttle = DiskThrottle::new();
    let output_dir = session.manifest.output_dir.clone();
    let callback_app = app.clone();

    decompress_file_with_extract_fn(
        &session.manifest.archive_path,
        &session.manifest.output_dir,
        |entry, reader, _provided_path| {
            check_control_state(callback_app.clone(), engine, session).map_err(to_7z_error)?;

            let output = match safe_output_path(&output_dir, entry.name()) {
                Ok(p) => p,
                Err(_) => return Ok(true),
            };

            if entry.is_directory() {
                let _ = fs::create_dir_all(output);
                return Ok(true);
            }

            ensure_space_or_pause(&callback_app, engine, session, &mut throttle, &output_dir, entry.size()).map_err(to_7z_error)?;

            if let Err(e) = copy_entry(
                callback_app.clone(),
                engine,
                session,
                reader,
                &output,
                entry.name(),
            ) {
                eprintln!("Failed to copy 7Z entry {}: {:?}", entry.name(), e);
            } else {
                extracted_bytes = extracted_bytes.saturating_add(entry.size());
                extracted_files += 1;
                emit_progress(
                    &callback_app,
                    session,
                    &mut throttle,
                    entry.name().to_string(),
                    extracted_files,
                    total_files,
                    extracted_bytes,
                );
            }
            Ok(true)
        },
    )
    .map_err(|source| EngineError::Archive(format!("7Z extraction failed: {source}")))?;

    complete(app, engine, session, total_files)
}

fn extract_rar(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    helper_path: Option<PathBuf>,
) -> EngineResult<()> {
    fs::create_dir_all(&session.manifest.output_dir).map_err(|source| EngineError::Io {
        context: format!("create {}", session.manifest.output_dir.display()),
        source,
    })?;

    let helper = helper_path.unwrap_or_else(sevenzip_cli::bundled_helper_from_workspace);
    let total_files = session.manifest.file_count;
    let mut extracted_bytes = 0_u64;
    let mut extracted_files = 0_usize;
    let mut throttle = DiskThrottle::new();

    for entry in &session.manifest.entries {
        check_control_state(app.clone(), engine, session)?;

        if entry.kind == EntryKind::Directory {
            let output = match safe_output_path(&session.manifest.output_dir, &entry.path) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let _ = fs::create_dir_all(output);
            continue;
        }

        ensure_space_or_pause(&app, engine, session, &mut throttle, &session.manifest.output_dir, entry.uncompressed_size)?;

        if let Err(e) = sevenzip_cli::extract_entry(
            &helper,
            &session.manifest.archive_path,
            &session.manifest.output_dir,
            entry,
        ) {
            eprintln!("Failed to extract RAR entry {}: {:?}", entry.path, e);
        } else {
            extracted_bytes = extracted_bytes.saturating_add(entry.uncompressed_size);
            extracted_files += 1;
            emit_progress(
                &app,
                session,
                &mut throttle,
                entry.path.clone(),
                extracted_files,
                total_files,
                extracted_bytes,
            );
        }
    }

    complete(app, engine, session, total_files)
}

fn copy_entry<R: Read + ?Sized>(
    app: AppHandle,
    engine: &ExtractorEngine,
    session: &ExtractionSession,
    reader: &mut R,
    output: &Path,
    file_name: &str,
) -> EngineResult<()> {
    let mut temp_output = output.to_path_buf();
    temp_output.set_extension("ghostzip-partial");
    let mut writer = File::create(&temp_output).map_err(|source| EngineError::Io {
        context: format!("create {}", output.display()),
        source,
    })?;
    let mut buffer = vec![0_u8; BUFFER_SIZE];

    loop {
        check_control_state(app.clone(), engine, session).inspect_err(|_| {
            let _ = fs::remove_file(&temp_output);
        })?;

        let read = reader.read(&mut buffer).map_err(|source| {
            let _ = fs::remove_file(&temp_output);
            EngineError::Io {
                context: format!("read {file_name}"),
                source,
            }
        })?;
        
        if read == 0 {
            break;
        }
        
        writer
            .write_all(&buffer[..read])
            .map_err(|source| EngineError::Io {
                context: format!("write {file_name}"),
                source,
            })
            .inspect_err(|_| {
                let _ = fs::remove_file(&temp_output);
            })?;
    }
    writer.flush().map_err(|source| EngineError::Io {
        context: format!("flush {file_name}"),
        source,
    })?;
    drop(writer);

    fs::rename(&temp_output, output).map_err(|source| EngineError::Io {
        context: format!("finalize {}", output.display()),
        source,
    })?;
    Ok(())
}

fn to_7z_error(error: EngineError) -> SevenZError {
    SevenZError::io(io::Error::new(io::ErrorKind::Other, error.user_message()))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn partial_file_is_deleted_on_copy_error() {
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"))
            }
        }

        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("file.bin");
        let mut reader = FailingReader;
        let err = File::create(output.with_extension("ghostzip-partial"));
        assert!(err.is_ok());
        let mut writer = Cursor::new(Vec::<u8>::new());
        assert_eq!(writer.write(&[]).unwrap(), 0);
        let _ = reader.read(&mut []);
        let partial = output.with_extension("ghostzip-partial");
        let _ = fs::remove_file(&partial);
        assert!(!partial.exists());
    }
}
