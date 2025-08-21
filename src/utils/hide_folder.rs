use std::path::PathBuf;
use std::io;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW, INVALID_FILE_ATTRIBUTES};

pub fn hide_folder(path: &PathBuf) -> io::Result<()> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path must be a directory",
        ));
    }

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if !file_name.starts_with('.') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Directory name must start with '.'",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid directory name",
        ));
    }

    hide_folder_impl(path)
}

#[cfg(windows)]
fn hide_folder_impl(path: &PathBuf) -> io::Result<()> {
    // Convert to Windows wide string format
    let path_str: Vec<u16> = path.as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();

    // Get current attributes
    let attrs = unsafe { GetFileAttributesW(path_str.as_ptr()) };
    if attrs == INVALID_FILE_ATTRIBUTES {
        return Err(io::Error::last_os_error());
    }

    // Add hidden attribute flag
    let new_attrs = attrs | winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

    // Set new attributes
    let success = unsafe { SetFileAttributesW(path_str.as_ptr(), new_attrs) };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

#[cfg(unix)]
fn hide_folder_impl(_path: &PathBuf) -> io::Result<()> {
    Ok(())
}

#[cfg(not(any(windows, unix)))]
fn hide_folder_impl(_path: &PathBuf) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Unsupported operating system",
    ))
}