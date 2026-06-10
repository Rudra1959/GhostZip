use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager, State, Emitter};

use crate::engine::{
    ArchiveManifest, DiskSpaceInfo, ExtractionMode, ExtractorEngine, UserExtractionMode,
};

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchContext {
    pub archive_path: Option<String>,
    pub output_dir: Option<String>,
    pub requested_action: String,
}

#[tauri::command]
pub fn get_launch_context() -> LaunchContext {
    let mut args = std::env::args().skip(1);
    let mut context = LaunchContext {
        requested_action: "open".to_string(),
        ..LaunchContext::default()
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--archive" => context.archive_path = args.next(),
            "--output" => context.output_dir = args.next(),
            "--action" => {
                if let Some(action) = args.next() {
                    context.requested_action = action;
                }
            }
            path if !path.starts_with("--") && context.archive_path.is_none() => {
                context.archive_path = Some(path.to_string());
            }
            _ => {}
        }
    }

    if context.output_dir.is_none() {
        if let Some(archive_path) = context.archive_path.as_deref() {
            context.output_dir = default_output_for_action(archive_path, &context.requested_action);
        }
    }

    context
}

#[tauri::command]
pub fn default_output_dir_for_archive(path: String) -> Result<String, String> {
    default_output_folder(&PathBuf::from(path))
        .ok_or_else(|| "Archive path does not have a parent directory.".to_string())
}

fn default_output_for_action(archive_path: &str, action: &str) -> Option<String> {
    let archive = PathBuf::from(archive_path);
    if action == "extract-here" {
        return archive.parent().map(|parent| parent.display().to_string());
    }
    default_output_folder(&archive)
}

fn default_output_folder(archive: &PathBuf) -> Option<String> {
    let parent = archive.parent()?;
    let file_name = archive.file_name().and_then(|name| name.to_str())?;
    let stem = archive_output_stem(file_name);
    Some(parent.join(stem).display().to_string())
}

fn archive_output_stem(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    for suffix in [
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".tbz2", ".txz", ".zip", ".7z", ".rar", ".tar",
        ".gz", ".bz2", ".xz",
    ] {
        if lower.ends_with(suffix) && file_name.len() > suffix.len() {
            return file_name[..file_name.len() - suffix.len()].to_string();
        }
    }

    file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| "Extracted".to_string())
}

#[cfg(test)]
mod tests {
    use super::archive_output_stem;

    #[test]
    fn default_output_folder_handles_common_archive_suffixes() {
        assert_eq!(archive_output_stem("photos.zip"), "photos");
        assert_eq!(archive_output_stem("backup.tar.gz"), "backup");
        assert_eq!(archive_output_stem("linux.tar.xz"), "linux");
        assert_eq!(archive_output_stem("game.7z"), "game");
        assert_eq!(archive_output_stem("movie.rar"), "movie");
    }
}

#[tauri::command]
pub fn inspect_archive(
    engine: State<'_, ExtractorEngine>,
    path: String,
    output_dir: String,
    app: AppHandle,
) -> Result<ArchiveManifest, String> {
    let helper_path = resolve_7za_helper(&app);
    engine
        .inspect_archive(PathBuf::from(path), PathBuf::from(output_dir), helper_path)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn start_extraction(
    app: AppHandle,
    engine: State<'_, ExtractorEngine>,
    session_id: String,
    mode: Option<UserExtractionMode>,
) -> Result<(), String> {
    let selected_mode = mode.unwrap_or(UserExtractionMode::Auto);
    let helper_path = resolve_7za_helper(&app);
    let engine_clone = engine.inner().clone();
    
    std::thread::spawn(move || {
        if let Err(e) = engine_clone.start_extraction(app.clone(), session_id, selected_mode, helper_path) {
            // Ignore send errors if the app is closing
            let _ = app.emit(
                crate::engine::EVENT_ERROR,
                e.user_message(),
            );
        }
    });

    Ok(())
}

#[tauri::command]
pub fn pause_extraction(
    engine: State<'_, ExtractorEngine>,
    session_id: String,
) -> Result<(), String> {
    engine
        .set_mode(&session_id, ExtractionMode::PausedByUser)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn resume_extraction(
    engine: State<'_, ExtractorEngine>,
    session_id: String,
) -> Result<(), String> {
    engine
        .set_mode(&session_id, ExtractionMode::Running)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn cancel_extraction(
    engine: State<'_, ExtractorEngine>,
    session_id: String,
) -> Result<(), String> {
    engine
        .set_mode(&session_id, ExtractionMode::Cancelled)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn get_disk_space(path: String) -> Result<DiskSpaceInfo, String> {
    ExtractorEngine::disk_space_for(PathBuf::from(path)).map_err(|error| error.user_message())
}

#[tauri::command]
pub fn delete_archive(path: String) -> Result<(), String> {
    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete archive: {}", e))
}

fn resolve_7za_helper(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .resource_dir()
        .ok()
        .map(|resource_dir| resource_dir.join("resources").join("7za.exe"))
        .filter(|path| path.exists())
        .or_else(|| {
            let dev_path = ExtractorEngine::bundled_helper_path();
            dev_path.exists().then_some(dev_path)
        })
}
