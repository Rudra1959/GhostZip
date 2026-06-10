use std::fs::File;
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Ioctl::{FSCTL_SET_SPARSE, FSCTL_SET_ZERO_DATA, FILE_ZERO_DATA_INFORMATION};
use windows_sys::Win32::System::IO::DeviceIoControl;

pub fn mark_sparse(file: &File) -> std::io::Result<()> {
    let handle = file.as_raw_handle() as HANDLE;
    let mut bytes_returned = 0;
    let success = unsafe {
        DeviceIoControl(
            handle,
            FSCTL_SET_SPARSE,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            0,
            &mut bytes_returned,
            std::ptr::null_mut(),
        )
    };
    if success == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn punch_hole(file: &File, offset: u64, length: u64) -> std::io::Result<()> {
    let handle = file.as_raw_handle() as HANDLE;
    let mut info = FILE_ZERO_DATA_INFORMATION {
        FileOffset: offset as i64,
        BeyondFinalZero: (offset + length) as i64,
    };
    let mut bytes_returned = 0;
    let success = unsafe {
        DeviceIoControl(
            handle,
            FSCTL_SET_ZERO_DATA,
            &mut info as *mut _ as *mut _,
            std::mem::size_of::<FILE_ZERO_DATA_INFORMATION>() as u32,
            std::ptr::null_mut(),
            0,
            &mut bytes_returned,
            std::ptr::null_mut(),
        )
    };
    if success == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
