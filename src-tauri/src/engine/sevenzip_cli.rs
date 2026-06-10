use std::path::{Path, PathBuf};
use std::process::Command;

use super::archive::{safe_output_path, ArchiveEntry, EntryKind};
use super::{EngineError, EngineResult};

pub fn bundled_helper_from_workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("node_modules")
        .join("7zip-bin")
        .join("win")
        .join("x64")
        .join("7za.exe")
}

pub fn inspect_archive(helper_path: &Path, archive_path: &Path) -> EngineResult<Vec<ArchiveEntry>> {
    let output = Command::new(helper_path)
        .args(["l", "-slt", "-ba", "-sccUTF-8"])
        .arg(archive_path)
        .output()
        .map_err(|source| EngineError::Io {
            context: format!("run 7-Zip helper {}", helper_path.display()),
            source,
        })?;

    if !output.status.success() {
        return Err(EngineError::Archive(helper_error(
            "RAR inspection",
            &output.stderr,
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_list_output(&stdout)
}

pub fn extract_entry(
    helper_path: &Path,
    archive_path: &Path,
    output_dir: &Path,
    entry: &ArchiveEntry,
) -> EngineResult<()> {
    safe_output_path(output_dir, &entry.path)?;
    let output_arg = format!("-o{}", output_dir.display());
    let output = Command::new(helper_path)
        .arg("x")
        .arg("-y")
        .arg("-bd")
        .arg("-bb0")
        .arg("-aoa")
        .arg("-sccUTF-8")
        .arg("-spd")
        .arg(output_arg)
        .arg(archive_path)
        .arg("--")
        .arg(&entry.path)
        .output()
        .map_err(|source| EngineError::Io {
            context: format!("run 7-Zip helper {}", helper_path.display()),
            source,
        })?;

    if !output.status.success() {
        return Err(EngineError::Archive(helper_error(
            &format!("RAR extraction failed for {}", entry.path),
            &output.stderr,
        )));
    }

    Ok(())
}

fn helper_error(context: &str, stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr);
    let trimmed = message.trim();
    if trimmed.is_empty() {
        format!("{context}: 7-Zip helper exited with an error")
    } else if trimmed.to_ascii_lowercase().contains("wrong password")
        || trimmed.to_ascii_lowercase().contains("encrypted")
    {
        format!("{context}: password required or password is incorrect")
    } else {
        format!("{context}: {trimmed}")
    }
}

fn parse_list_output(output: &str) -> EngineResult<Vec<ArchiveEntry>> {
    let mut entries = Vec::new();
    let mut path: Option<String> = None;
    let mut size = 0_u64;
    let mut packed_size = 0_u64;
    let mut attributes = String::new();

    for line in output.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if let Some(path) = path.take() {
                if !path.is_empty() {
                    entries.push(ArchiveEntry {
                        path,
                        compressed_size: packed_size,
                        uncompressed_size: size,
                        kind: if attributes.contains('D') {
                            EntryKind::Directory
                        } else {
                            EntryKind::File
                        },
                    });
                }
            }
            size = 0;
            packed_size = 0;
            attributes.clear();
            continue;
        }

        if let Some((key, value)) = line.split_once(" = ") {
            match key {
                "Path" => path = Some(value.to_string()),
                "Size" => size = value.parse().unwrap_or(0),
                "Packed Size" => packed_size = value.parse().unwrap_or(0),
                "Attributes" => attributes = value.to_string(),
                _ => {}
            }
        }
    }

    if entries.is_empty() {
        return Err(EngineError::Archive(
            "RAR inspection did not return any extractable entries.".to_string(),
        ));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_7zip_technical_listing() {
        let output = r#"
Path = folder
Size = 0
Packed Size = 0
Attributes = D

Path = folder/file.txt
Size = 12
Packed Size = 8
Attributes = A
"#;

        let entries = parse_list_output(output).expect("parse");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, EntryKind::Directory);
        assert_eq!(entries[1].path, "folder/file.txt");
        assert_eq!(entries[1].uncompressed_size, 12);
    }
}
