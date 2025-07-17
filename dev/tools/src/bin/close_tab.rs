//! Dim a window to a given opacity

use clap::Parser;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use platform::web;
use tools::filters::{
    ProcessFilter, ProcessWindowGroup, WindowDetails, WindowFilter, match_running_windows,
};
use util::error::Result;
use util::{Target, config};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Window title text to filter by
    #[arg()]
    title: Option<String>,
}

fn main() -> Result<()> {
    util::set_target(Target::Tool {
        name: "close_tab".to_string(),
    });
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    let matches = match_running_windows(
        &WindowFilter {
            title: args.title,
            ..Default::default()
        },
        &ProcessFilter {
            name: Some("Google Chrome".to_string()),
            ..Default::default()
        },
    )?;

    let detect = web::Detect::new()?;
    if let Some(details) = select_window(&matches)? {
        let element = detect.get_chromium_element(&details.window)?;
        detect.close_current_tab(&element)?;
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
