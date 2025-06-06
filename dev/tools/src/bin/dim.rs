//! Dim a window to a given opacity

use std::collections::HashMap;
use std::path::Path;

use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use platform::objects::{FileVersionInfo, Process, ProcessId, ProcessThreadId, Window};
use util::error::Result;
use util::{config, Target};
use windows::core::HRESULT;
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Window opacity (0.0 to 1.0)
    #[arg(short, long, value_parser = |s: &str| -> Result<f64, String> {
        let val = s.parse::<f64>().map_err(|_| "Invalid number".to_string())?;
        if val >= 0.0 && val <= 1.0 {
            Ok(val)
        } else {
            Err("Opacity must be between 0.0 and 1.0".to_string())
        }
    })]
    opacity: f64,

    /// Application name to filter by
    #[arg(long)]
    app: Option<String>,

    /// Process ID to filter by
    #[arg(long)]
    pid: Option<ProcessId>,

    /// Window handle in hexadecimal
    #[arg(long)]
    handle: Option<String>,

    /// Window title text to filter by
    #[arg()]
    title: Option<String>,
}

fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    let matches = match_window(
        &WindowFilter {
            handle: args.handle,
            title: args.title,
        },
        &ProcessFilter {
            pid: args.pid,
            name: args.app,
        },
    )?;

    if let Some(details) = select_window(&matches)? {
        details.window.dim(args.opacity)?;
    } else {
        eprintln!("No windows found matching the criteria");
    }

    Ok(())
}

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
}

#[derive(Clone, Debug)]
struct ProcessWindowGroup {
    pub process: ProcessDetails,
    pub windows: Vec<WindowDetails>,
}

#[derive(Clone, Debug)]

struct ProcessDetails {
    pub pid: ProcessId,
    #[allow(dead_code)]
    pub path: String,
    pub name: String,
}

#[derive(Clone, Debug)]
struct WindowDetails {
    pub window: Window,
    pub title: String,
    pub ptid: ProcessThreadId,
}

fn get_process_details(pid: ProcessId) -> Result<Option<ProcessDetails>> {
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

fn match_window(
    window_filter: &WindowFilter,
    process_filter: &ProcessFilter,
) -> Result<Vec<ProcessWindowGroup>> {
    let mut windows = Window::get_all_visible_windows()?
        .into_iter()
        .map(|window| {
            Ok(WindowDetails {
                title: window.title()?,
                ptid: window.ptid()?,
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

fn select_window(groups: &[ProcessWindowGroup]) -> Result<Option<&WindowDetails>> {
    if groups.is_empty() {
        return Ok(None);
    }

    // Flatten all windows into a single list with process info
    let mut all_windows = Vec::new();
    for group in groups {
        for window in &group.windows {
            all_windows.push((group, window));
        }
    }

    if all_windows.is_empty() {
        return Ok(None);
    }

    if all_windows.len() == 1 {
        return Ok(Some(all_windows[0].1));
    }

    // Create selection items
    let items: Vec<String> = all_windows
        .iter()
        .map(|(group, window)| {
            format!(
                "{} (pid: {}) - \"{}\" (hwnd: {:08x})",
                group.process.name, group.process.pid, window.title, window.window.hwnd.0
            )
        })
        .collect();

    // Show selection prompt
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a window to dim")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(Some(all_windows[selection].1))
}
