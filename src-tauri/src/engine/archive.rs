use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sevenz_rust::Archive as SevenZArchive;
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use super::{sevenzip_cli, DiskSpaceInfo, EngineError, EngineResult};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    Zip,
    Rar,
    SevenZip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
}

impl ArchiveFormat {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "ZIP",
            ArchiveFormat::Rar => "RAR",
            ArchiveFormat::SevenZip => "7Z",
            ArchiveFormat::Tar => "TAR",
            ArchiveFormat::TarGz => "TAR.GZ",
            ArchiveFormat::TarBz2 => "TAR.BZ2",
            ArchiveFormat::TarXz => "TAR.XZ",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl EntryKind {
    pub fn is_extractable(self) -> bool {
        matches!(self, EntryKind::File | EntryKind::Directory)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntry {
    pub path: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub kind: EntryKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveManifest {
    pub session_id: String,
    pub archive_path: PathBuf,
    pub output_dir: PathBuf,
    pub format: ArchiveFormat,
    pub file_count: usize,
    pub directory_count: usize,
    pub compressed_size: u64,
    pub total_uncompressed_size: u64,
    pub disk: DiskSpaceInfo,
    pub recommended_space_saver: bool,
    pub tight_space: bool,
    pub entries: Vec<ArchiveEntry>,
}

pub fn detect_archive_format(path: &Path) -> EngineResult<ArchiveFormat> {
    let mut file = File::open(path).map_err(|source| EngineError::Io {
        context: format!("open archive {}", path.display()),
        source,
    })?;
    let mut header = [0_u8; 560];
    let read = file.read(&mut header).map_err(|source| EngineError::Io {
        context: format!("read archive header {}", path.display()),
        source,
    })?;
    let header = &header[..read];

    if header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06") {
        return Ok(ArchiveFormat::Zip);
    }
    if header.starts_with(b"Rar!\x1A\x07\x00") || header.starts_with(b"Rar!\x1A\x07\x01\x00") {
        return Ok(ArchiveFormat::Rar);
    }
    if header.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
        return Ok(ArchiveFormat::SevenZip);
    }
    if header.starts_with(&[0x1F, 0x8B]) {
        return Ok(ArchiveFormat::TarGz);
    }
    if header.starts_with(b"BZh") {
        return Ok(ArchiveFormat::TarBz2);
    }
    if header.starts_with(&[0xFD, b'7', b'z', b'X', b'Z', 0x00]) {
        return Ok(ArchiveFormat::TarXz);
    }
    if header.len() > 262 && &header[257..262] == b"ustar" {
        return Ok(ArchiveFormat::Tar);
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "tar" => Ok(ArchiveFormat::Tar),
        _ => Err(EngineError::UnsupportedFormat(extension)),
    }
}

pub fn inspect_entries(
    path: &Path,
    format: ArchiveFormat,
    helper_path: Option<&Path>,
) -> EngineResult<Vec<ArchiveEntry>> {
    match format {
        ArchiveFormat::Zip => inspect_zip(path),
        ArchiveFormat::Tar => inspect_tar(File::open(path).map_err(|source| EngineError::Io {
            context: format!("open tar archive {}", path.display()),
            source,
        })?),
        ArchiveFormat::TarGz => {
            inspect_tar(GzDecoder::new(File::open(path).map_err(|source| {
                EngineError::Io {
                    context: format!("open gzip archive {}", path.display()),
                    source,
                }
            })?))
        }
        ArchiveFormat::TarBz2 => inspect_tar(bzip2::read::BzDecoder::new(
            File::open(path).map_err(|source| EngineError::Io {
                context: format!("open bzip2 archive {}", path.display()),
                source,
            })?,
        )),
        ArchiveFormat::TarXz => {
            inspect_tar(XzDecoder::new(File::open(path).map_err(|source| {
                EngineError::Io {
                    context: format!("open xz archive {}", path.display()),
                    source,
                }
            })?))
        }
        ArchiveFormat::Rar => {
            let helper = helper_path
                .map(Path::to_path_buf)
                .unwrap_or_else(sevenzip_cli::bundled_helper_from_workspace);
            sevenzip_cli::inspect_archive(&helper, path)
        }
        ArchiveFormat::SevenZip => inspect_7z(path),
    }
}

fn inspect_7z(path: &Path) -> EngineResult<Vec<ArchiveEntry>> {
    let mut file = File::open(path).map_err(|source| EngineError::Io {
        context: format!("open 7z archive {}", path.display()),
        source,
    })?;
    let file_len = file
        .metadata()
        .map_err(|source| EngineError::Io {
            context: format!("read 7z metadata {}", path.display()),
            source,
        })?
        .len();
    let archive = SevenZArchive::read(&mut file, file_len, &[])
        .map_err(|source| EngineError::Archive(format!("7Z inspection failed: {source}")))?;

    Ok(archive
        .files
        .iter()
        .map(|file| ArchiveEntry {
            path: file.name().to_string(),
            compressed_size: file.compressed_size,
            uncompressed_size: file.size,
            kind: if file.is_directory() {
                EntryKind::Directory
            } else {
                EntryKind::File
            },
        })
        .collect())
}

fn inspect_zip(path: &Path) -> EngineResult<Vec<ArchiveEntry>> {
    let file = File::open(path).map_err(|source| EngineError::Io {
        context: format!("open zip archive {}", path.display()),
        source,
    })?;
    let mut archive = ZipArchive::new(file)
        .map_err(|source| EngineError::Archive(format!("ZIP inspection failed: {source}")))?;
    let mut entries = Vec::with_capacity(archive.len());

    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|source| {
            EngineError::Archive(format!("ZIP entry {index} failed: {source}"))
        })?;
        let name = file.name().to_string();
        let kind = if file.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };
        entries.push(ArchiveEntry {
            path: name,
            compressed_size: file.compressed_size(),
            uncompressed_size: file.size(),
            kind,
        });
    }
    Ok(entries)
}

fn inspect_tar<R: Read>(reader: R) -> EngineResult<Vec<ArchiveEntry>> {
    let mut archive = TarArchive::new(reader);
    let entries = archive
        .entries()
        .map_err(|source| EngineError::Archive(format!("TAR inspection failed: {source}")))?;
    let mut manifest = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| {
            EngineError::Archive(format!("TAR entry inspection failed: {source}"))
        })?;
        let header = entry.header();
        let kind = match header.entry_type() {
            tar::EntryType::Regular => EntryKind::File,
            tar::EntryType::Directory => EntryKind::Directory,
            tar::EntryType::Symlink => EntryKind::Symlink,
            _ => EntryKind::Other,
        };
        let path = entry
            .path()
            .map_err(|source| EngineError::Archive(format!("TAR path read failed: {source}")))?
            .to_string_lossy()
            .to_string();
        manifest.push(ArchiveEntry {
            path,
            compressed_size: 0,
            uncompressed_size: header.size().unwrap_or(0),
            kind,
        });
    }
    Ok(manifest)
}

pub fn safe_output_path(output_dir: &Path, entry_path: &str) -> EngineResult<PathBuf> {
    let relative = Path::new(entry_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(EngineError::InvalidArchivePath(entry_path.to_string()));
    }

    let base = std::fs::canonicalize(output_dir).map_err(|source| EngineError::Io {
        context: format!("canonicalize output directory {}", output_dir.display()),
        source,
    })?;
    let target = base.join(relative);
    let parent = target.parent().unwrap_or(&base);
    std::fs::create_dir_all(parent).map_err(|source| EngineError::Io {
        context: format!("create output parent {}", parent.display()),
        source,
    })?;
    let canonical_parent = std::fs::canonicalize(parent).map_err(|source| EngineError::Io {
        context: format!("canonicalize output parent {}", parent.display()),
        source,
    })?;

    if !canonical_parent.starts_with(&base) {
        return Err(EngineError::InvalidArchivePath(entry_path.to_string()));
    }

    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn rejects_path_traversal() {
        let dir = tempdir().expect("tempdir");
        let result = safe_output_path(dir.path(), "../../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn detects_magic_bytes_without_extension() {
        let dir = tempdir().expect("tempdir");
        let zip = dir.path().join("archive.bin");
        let rar = dir.path().join("rar.bin");
        let sevenz = dir.path().join("seven.bin");
        let targz = dir.path().join("gzip.bin");
        std::fs::write(&zip, b"PK\x03\x04rest").expect("zip");
        std::fs::write(&rar, b"Rar!\x1A\x07\x00rest").expect("rar");
        std::fs::write(&sevenz, [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]).expect("7z");
        let mut file = File::create(&targz).expect("gz");
        file.write_all(&[0x1F, 0x8B, 0x08]).expect("write gz");

        assert_eq!(detect_archive_format(&zip).unwrap(), ArchiveFormat::Zip);
        assert_eq!(detect_archive_format(&rar).unwrap(), ArchiveFormat::Rar);
        assert_eq!(
            detect_archive_format(&sevenz).unwrap(),
            ArchiveFormat::SevenZip
        );
        assert_eq!(detect_archive_format(&targz).unwrap(), ArchiveFormat::TarGz);
    }
}
