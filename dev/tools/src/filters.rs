use std::collections::HashMap;
use std::path::Path;

use platform::objects::{FileVersionInfo, Process, ProcessId, ProcessThreadId, Window};
use util::error::Result;
use windows::core::HRESULT;
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

/// Filter for a process
pub struct ProcessFilter {
    /// Process ID
    pub pid: Option<ProcessId>,
    /// Process name to filter by
    pub name: Option<String>,
}

/// Filter for a window
#[derive(Debug, Clone)]
pub struct WindowFilter {
    /// Window handle in hexadecimal
    pub handle: Option<String>,
    /// Window title text to filter by
    pub title: Option<String>,
    /// Application User Model ID to filter by
    pub aumid: Option<String>,
}

/// A group of windows and their process
#[derive(Clone, Debug)]
pub struct ProcessWindowGroup {
    /// The process
    pub process: ProcessDetails,
    /// The windows
    pub windows: Vec<WindowDetails>,
}

/// The details of a process
#[derive(Clone, Debug)]

pub struct ProcessDetails {
    /// The process ID
    pub pid: ProcessId,
    /// The path to the process
    #[allow(dead_code)]
    pub path: String,
    /// The name of the process (derived from the file name and version info)
    pub name: String,
}

/// The details of a window
#[derive(Clone, Debug)]
pub struct WindowDetails {
    /// The window
    pub window: Window,
    /// The title of the window
    pub title: String,
    /// The Application User Model ID of the window
    pub aumid: Option<String>,
    /// The process thread ID of the window
    pub ptid: ProcessThreadId,
}

/// Match windows and processes based on the filters
pub fn match_running_windows(
    window_filter: &WindowFilter,
    process_filter: &ProcessFilter,
) -> Result<Vec<ProcessWindowGroup>> {
    let mut windows = Window::get_all_visible_windows()?
        .into_iter()
        .map(|window| {
            Ok(WindowDetails {
                title: window.title()?,
                ptid: window.ptid()?,
                aumid: window.aumid().ok(),
                window,
            })
        })
        .collect::<Result<Vec<WindowDetails>>>()?;

    if let Some(handle_filter) = &window_filter.handle {
        let handle_filter = handle_filter.to_lowercase();
        windows.retain(|w| {
            format!("{:08x}", w.window.hwnd.0)
                .to_lowercase()
                .contains(&handle_filter)
        });
    }
    if let Some(aumid_filter) = &window_filter.aumid {
        let aumid_filter = aumid_filter.to_lowercase();
        windows.retain(|w| {
            w.aumid
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_lowercase()
                .contains(&aumid_filter)
        });
    }
    if let Some(title_filter) = &window_filter.title {
        let title_filter = title_filter.to_lowercase();
        windows.retain(|w| w.title.to_lowercase().contains(&title_filter));
    }

    let mut window_groups = HashMap::new();
    for window in windows {
        window_groups
            .entry(window.ptid.pid)
            .or_insert(Vec::new())
            .push(window);
    }

    let mut matches: Vec<ProcessWindowGroup> = Vec::new();

    for (pid, windows) in window_groups {
        let process = get_process_details(pid)?;
        if let Some(process) = process {
            if let Some(pid_filter) = &process_filter.pid {
                if pid != *pid_filter {
                    continue;
                }
            }

            if let Some(name_filter) = &process_filter.name {
                if !process
                    .name
                    .to_lowercase()
                    .contains(&name_filter.to_lowercase())
                {
                    continue;
                }
            }
            matches.push(ProcessWindowGroup { process, windows });
        }
    }

    Ok(matches)
}

/// Get the details of a process
pub fn get_process_details(pid: ProcessId) -> Result<Option<ProcessDetails>> {
    let process = match Process::new(pid) {
        Ok(process) => process,
        Err(e) => {
            if e.downcast_ref::<windows::core::Error>().unwrap().code()
                == HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
            {
                return Ok(None);
            }
            return Err(e);
        }
    };

    let path = process.path()?;
    let file_name = Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let name = FileVersionInfo::new(&path)
        .and_then(|mut vs| {
            let mut file_description = vs.query_value("FileDescription")?;
            if file_description.ends_with(".exe") {
                let product_name = vs.query_value("ProductName")?;
                file_description = product_name;
            }
            if file_description.is_empty() {
                file_description = file_name.clone();
            }
            Ok(file_description)
        })
        .ok()
        .unwrap_or(file_name);
    Ok(Some(ProcessDetails { pid, path, name }))
}
