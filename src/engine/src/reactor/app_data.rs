use super::*;
use crate::errors::*;
use crate::os::prelude::*;

#[derive(Debug)]
pub struct AppData {
    pub exit_watcher: ProcessExit,
    pub process: Process,
    pub app: App,
    pub arguments: String,
}

impl AppData {
    pub fn create(
        state: &mut State,
        reactor: &Reactor,
        window: Window,
        pid: ProcessId,
    ) -> Result<Self> {
        let process = Process::new(pid, default())?;
        let exit_watcher = ProcessExit::watch(&process, state.process_exits.0.clone())?;
        let (path, cmdline) = process.path_and_cmdline()?;
        let mut app = state.create_app(reactor, window, &process, &path, &cmdline)?;
        info!("NEW: {:?}", app);
        state.db.insert_app(&mut app)?;
        Ok(AppData {
            exit_watcher,
            process,
            app,
            arguments: cmdline,
        })
    }
}
