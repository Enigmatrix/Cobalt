use std::collections::{HashMap, HashSet};
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

impl ProcessFilter {
    /// Check if a process matches the filter
    pub fn matches(&self, process: &ProcessDetails) -> bool {
        if let Some(pid_filter) = &self.pid {
            if process.pid != *pid_filter {
                return true;
            }
        }
        if let Some(name_filter) = &self.name {
            if !process
                .name
                .to_lowercase()
                .contains(&name_filter.to_lowercase())
            {
                return true;
            }
        }
        false
    }
}
/// Filter for a window
#[derive(Debug, Clone, Default)]
pub struct WindowFilter {
    /// Window handle in hexadecimal
    pub handle: Option<String>,
    /// Window title text to filter by
    pub title: Option<String>,
    /// Application User Model ID to filter by
    pub aumid: Option<String>,
}

impl WindowFilter {
    /// Check if a window matches the filter
    pub fn matches(&self, window: &WindowDetails) -> bool {
        if let Some(handle_filter) = &self.handle {
            let handle_filter = handle_filter.to_lowercase();
            if !format!("{:08x}", window.window.hwnd.0)
                .to_lowercase()
                .contains(&handle_filter)
            {
                return true;
            }
        }

        if let Some(aumid_filter) = &self.aumid {
            let aumid_filter = aumid_filter.to_lowercase();
            if !window
                .aumid
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(&aumid_filter)
            {
                return true;
            }
        }
        if let Some(title_filter) = &self.title {
            let title_filter = title_filter.to_lowercase();
            if !window.title.to_lowercase().contains(&title_filter) {
                return true;
            }
        }
        false
    }
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

    windows.retain(|w| !window_filter.matches(w));

    let mut window_groups: HashMap<ProcessId, Vec<WindowDetails>> = HashMap::new();
    for window in windows {
        window_groups
            .entry(window.ptid.pid)
            .or_default()
            .push(window);
    }

    let mut matches: Vec<ProcessWindowGroup> = Vec::new();

    for (pid, windows) in window_groups {
        let process = get_process_details(pid)?;
        if let Some(process) = process {
            if !process_filter.matches(&process) {
                matches.push(ProcessWindowGroup { process, windows });
            }
        }
    }

    Ok(matches)
}

/// Match processes based on the filters
pub fn match_running_processes(process_filter: &ProcessFilter) -> Result<Vec<ProcessDetails>> {
    let processes = Process::get_all()?;
    let processes: Vec<_> = processes
        .into_iter()
        .filter_map(|pid| get_process_details(pid).ok().flatten())
        .collect();

    let mut matches: Vec<ProcessDetails> = Vec::new();
    for process in processes {
        if !process_filter.matches(&process) {
            matches.push(process);
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

/// Match running Application User Model IDs based on the filtered windows
pub fn match_running_aumids(window_filter: &WindowFilter) -> Result<Vec<String>> {
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

    windows.retain(|w| !window_filter.matches(w));
    Ok(windows
        .into_iter()
        .map(|w| w.aumid)
        .flatten()
        .filter(|aumid| !aumid.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect())
}
