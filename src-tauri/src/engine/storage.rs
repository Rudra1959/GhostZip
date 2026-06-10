use std::path::Path;

use serde::{Deserialize, Serialize};
use sysinfo::Disks;

use super::{EngineError, EngineResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpaceInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

pub fn disk_space_for(path: &Path) -> EngineResult<DiskSpaceInfo> {
    let target = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .iter()
        .filter(|disk| target.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .ok_or_else(|| {
            EngineError::Archive(format!("No mounted drive found for {}", path.display()))
        })?;

    Ok(DiskSpaceInfo {
        mount_point: disk.mount_point().display().to_string(),
        total_bytes: disk.total_space(),
        available_bytes: disk.available_space(),
    })
}
