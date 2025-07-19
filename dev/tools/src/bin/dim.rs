//! Dim a window to a given opacity

use clap::Parser;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use platform::objects::{ProcessId, Window};
use tools::filters::{
    ProcessFilter, ProcessWindowGroup, WindowDetails, WindowFilter, match_running_windows,
};
use util::error::Result;
use util::{Target, config};
use windows::Win32::Foundation::HWND;

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

    /// Application User Model ID to filter by
    #[arg(long)]
    aumid: Option<String>,

    /// Window title text to filter by
    #[arg()]
    title: Option<String>,
}

fn main() -> Result<()> {
    util::set_target(Target::Tool {
        name: "dim".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    if let Some(handle) = args.handle {
        let handle = u32::from_str_radix(&handle, 16)?;
        let window = Window::new(HWND(handle as _));
        window.dim(args.opacity)?;
        return Ok(());
    }
    let matches = match_running_windows(
        &WindowFilter {
            handle: args.handle,
            title: args.title,
            aumid: args.aumid,
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
                group.process.name, group.process.pid, window.title, window.window.hwnd.0 as usize
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
