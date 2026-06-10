use super::{ArchiveEntry, EntryKind};

pub fn schedule_entries(entries: &[ArchiveEntry], space_saver: bool) -> Vec<ArchiveEntry> {
    let mut ordered = entries.to_vec();
    if space_saver {
        ordered.sort_by(|left, right| match (left.kind, right.kind) {
            (EntryKind::Directory, EntryKind::File) => std::cmp::Ordering::Less,
            (EntryKind::File, EntryKind::Directory) => std::cmp::Ordering::Greater,
            _ => left
                .uncompressed_size
                .cmp(&right.uncompressed_size)
                .then_with(|| left.path.cmp(&right.path)),
        });
    }
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_smallest_first_in_space_saver_mode() {
        let entries = vec![
            ArchiveEntry {
                path: "large.bin".to_string(),
                compressed_size: 1,
                uncompressed_size: 2_000,
                kind: EntryKind::File,
            },
            ArchiveEntry {
                path: "small.bin".to_string(),
                compressed_size: 1,
                uncompressed_size: 10,
                kind: EntryKind::File,
            },
        ];

        let ordered = schedule_entries(&entries, true);
        assert_eq!(ordered[0].path, "small.bin");
    }
}
