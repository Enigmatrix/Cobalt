use super::*;
use crate::data::db::Database;
use crate::data::model;

#[derive(Debug)]
pub struct AppInfo {
    // exited_watcher: process_exit::Watcher,
    pub process: Process,
    pub arguments: Option<String>,
    pub app: model::App,
}

impl AppInfo {
    pub fn get<'a>(
        window: &Window,
        msger: &Messenger,
        db: &mut Database,
        apps: &'a mut AppCache,
    ) -> Result<&'a mut AppInfo> {
        let (pid, _) = window.pid_tid()?;
        Ok(match apps.entry(pid) {
            Occupied(occ) => occ.into_mut(),
            Vacant(vac) => vac.insert(
                AppInfo::new(
                    window,
                    msger,
                    Process::new(pid, ProcessOptions::default())?,
                    db,
                )
                .with_context(|| "Create new AppInfo")?,
            ),
        })
    }

    fn new(
        window: &Window,
        msger: &Messenger,
        process: Process,
        db: &mut Database,
    ) -> Result<AppInfo> {
        let arguments = process.cmd().ok();
        let app = AppInfo::find_or_create_app(window, &process, db)
            .with_context(|| "Find/creating App for process")?;
        Ok(AppInfo {
            process,
            arguments,
            app,
        })
    }

    fn find_or_create_app(
        window: &Window,
        process: &Process,
        db: &mut Database,
    ) -> Result<model::App> {
        // TODO FOR TESTING PURPOSES
        Ok(model::App {
            id: 0,
            name: "dummy_name".to_string(),
            description: "dummy_description".to_string(),
            color: "red".to_string(),
            identity: model::AppIdentity::Win32 {
                path: "Some Win32 Path".to_string(),
            },
        })
    }

    fn get_identity(window: &Window, process: &Process) -> Result<model::AppIdentity> {
        let path = process.path().with_context(|| "Getting path of process")?;
        if window.is_uwp(process, &path) {
            Ok(model::AppIdentity::UWP {
                aumid: window.aumid().with_context(|| "Getting AUMID of Window")?,
            })
        } else {
            Ok(model::AppIdentity::Win32 { path })
        }
    }
}
