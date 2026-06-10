pub const EVENT_PROGRESS: &str = "extraction-progress";
pub const EVENT_COMPLETE: &str = "extraction-complete";
pub const EVENT_PAUSED: &str = "extraction-paused";
pub const EVENT_ERROR: &str = "extraction-error";

pub const BUFFER_SIZE: usize = 64 * 1024;
pub const LOW_SPACE_FLOOR_BYTES: u64 = 32 * 1024 * 1024;

pub const DISK_CHECK_INTERVAL_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
pub const DISK_CHECK_INTERVAL_FILES: usize = 50;
