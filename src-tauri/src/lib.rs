mod commands;
mod engine;

use commands::{
    cancel_extraction, default_output_dir_for_archive, get_disk_space, get_launch_context,
    inspect_archive, pause_extraction, resume_extraction, start_extraction, delete_archive,
};
use engine::ExtractorEngine;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(ExtractorEngine::default())
        .invoke_handler(tauri::generate_handler![
            get_launch_context,
            default_output_dir_for_archive,
            inspect_archive,
            start_extraction,
            pause_extraction,
            resume_extraction,
            cancel_extraction,
            get_disk_space,
            delete_archive
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GhostZip");
}
