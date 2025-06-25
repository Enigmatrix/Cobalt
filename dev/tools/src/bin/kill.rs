//! Dim a window to a given opacity

use clap::{ArgGroup, Parser};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use platform::objects::{Process, ProcessId, Window};
use tools::filters::{
    match_running_aumids, match_running_processes, ProcessDetails, ProcessFilter, WindowFilter,
};
use util::error::Result;
use util::{config, future as tokio, Target};
use windows::Win32::Foundation::HWND;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("process_filter")
        .multiple(true)
))]
struct Args {
    /// Application name to filter by
    #[arg(long, group = "process_filter")]
    app: Option<String>,

    /// Process ID to filter by
    #[arg(long, group = "process_filter")]
    pid: Option<ProcessId>,

    /// Application User Model ID to filter by
    #[arg(long, conflicts_with = "process_filter")]
    aumid: Option<Option<String>>,

    /// Window handle in hexadecimal
    #[arg(long)]
    handle: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    util::set_target(Target::Engine);
    let config = config::get_config()?;
    util::setup(&config)?;
    platform::setup()?;

    let args = Args::parse();
    if let Some(handle) = args.handle {
        let handle = u32::from_str_radix(&handle, 16)?;
        let window = Window::new(HWND(handle as _));

        let ptid = window.ptid()?;
        Process::new_killable(ptid.pid)?.kill(None)?;
        return Ok(());
    }
    if let Some(aumid) = args.aumid {
        let aumids = match_running_aumids(&WindowFilter {
            aumid,
            ..Default::default()
        })?;
        let aumid = select_aumid(&aumids)?;
        if let Some(aumid) = aumid {
            Process::kill_uwp(aumid).await?;
        }
    } else {
        let processes = match_running_processes(&ProcessFilter {
            pid: args.pid,
            name: args.app,
        })?;
        let process = select_process(&processes)?;
        if let Some(process) = process {
            Process::new_killable(process.pid)?.kill(Some(&process.path))?;
        }
    }

    Ok(())
}

fn select_process(processes: &[ProcessDetails]) -> Result<Option<&ProcessDetails>> {
    if processes.is_empty() {
        return Ok(None);
    }
    if processes.len() == 1 {
        return Ok(Some(&processes[0]));
    }

    // Create selection items
    let items: Vec<String> = processes
        .iter()
        .map(|process| format!("{} (pid: {})", process.name, process.pid))
        .collect();

    // Show selection prompt
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a process to kill")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(Some(&processes[selection]))
}

fn select_aumid(aumids: &[String]) -> Result<Option<&String>> {
    if aumids.is_empty() {
        return Ok(None);
    }
    if aumids.len() == 1 {
        return Ok(Some(&aumids[0]));
    }

    // Create selection items
    let items: Vec<String> = aumids.iter().map(|aumid| aumid.to_string()).collect();

    // Show selection prompt
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an AUMID to kill")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(Some(&aumids[selection]))
}
