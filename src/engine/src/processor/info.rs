use super::*;
use crate::data::db::Database;
use crate::data::model;
use util::futures::task;

#[derive(Debug)]
pub struct AppInfo {
    pub exited_watcher: process_exit::Watcher,
    pub process: Process,
    pub arguments: Option<String>,
    pub app: model::App,
}

#[derive(Debug)]
pub struct SessionInfo {
    pub closed_watcher: window_closed::Watcher,
    pub session: model::Session,
    pub pid: ProcessId,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub sess_id: model::Id,
    pub app_id: model::Id,
}

impl Info {
    #[log::instrument(skip(window, tx, db, sessions, apps))]
    pub fn from(
        window: &Window,
        tx: &ProcessorTx,
        db: &mut Database,
        sessions: &mut SessionCache,
        apps: &mut AppCache,
    ) -> Result<Info> {
        let (sess_info, app_info) = Info::session_info(window, tx, db, sessions, apps)
            .with_context(|| "Get SessionInfo & AppInfo")?;
        Ok(Info {
            sess_id: sess_info.session.id,
            app_id: app_info.app.id,
        })
    }

    #[log::instrument(skip(window, tx, db, sessions, apps))]
    fn session_info<'a>(
        window: &'a Window,
        tx: &'a ProcessorTx,
        db: &'a mut Database,
        sessions: &'a mut SessionCache,
        apps: &'a mut AppCache,
    ) -> Result<(&'a mut SessionInfo, &'a mut AppInfo)> {
        let vac = match sessions.entry(window.clone()) {
            Occupied(occ) => {
                let sess_info = occ.into_mut();
                let app_info = apps.get_mut(&sess_info.pid).unwrap(); // if the session exists, the app for it exists.
                log::trace!(
                    ?sess_info,
                    "using pre-existing SessionInfo from SessionCache"
                );
                return Ok((sess_info, app_info));
            }
            Vacant(vac) => vac,
        };

        let (pid, _) = window.pid_tid()?;
        let title = window
            .title()
            .with_context(|| "Get title of Window for Session")?;

        let closed_watcher = window_closed::Watcher::new(window, {
            let tx = tx.clone();
            move |window| {
                tx.send(Message::WindowClosed { window })
                    .with_context(|| "Send WindowClosed message")
            }
        })
        .with_context(|| "Create Window closed watcher for new SessionInfo")?;

        let app_info = Info::app_info(window, tx, db, apps).with_context(|| "Getting AppInfo")?;

        // always create a new session, no need to access db.
        let mut session = model::Session {
            id: 0,
            app_id: app_info.app.id,
            arguments: app_info.arguments.clone(),
            title,
        };
        log::trace!(?session, "newly created Session");

        db.insert_session(&mut session)
            .with_context(|| "Saving Session to Database")?;
        let sess_info = vac.insert(SessionInfo {
            closed_watcher,
            session,
            pid,
        });

        log::trace!(?sess_info, "newly created SessionInfo");

        Ok((sess_info, app_info))
    }

    #[log::instrument(skip(window, tx, db, apps))]
    pub fn app_info<'a>(
        window: &Window,
        tx: &ProcessorTx,
        db: &mut Database,
        apps: &'a mut AppCache,
    ) -> Result<&'a mut AppInfo> {
        let (pid, _) = window.pid_tid()?;

        let vac = match apps.entry(pid) {
            Occupied(occ) => {
                let app_info = occ.into_mut();
                log::trace!(?app_info, "using pre-existing AppInfo from AppCache");
                return Ok(app_info);
            }
            Vacant(vac) => vac,
        };

        let process =
            Process::new(pid, ProcessOptions::default()).with_context(|| "Create Process")?;
        let arguments = process.cmd().ok();
        let app = Info::find_or_create_app(window, &process, db, tx)
            .with_context(|| "Find/creating App for process")?;

        let exited_watcher = process_exit::Watcher::new(&process, {
            let tx = tx.clone();
            move |pid| {
                tx.send(Message::ProcessExit { pid })
                    .with_context(|| "Send ProcessExit message")
            }
        })
        .with_context(|| "Creating process exit watcher for AppInfo")?;

        let app_info = vac.insert(AppInfo {
            exited_watcher,
            process,
            arguments,
            app,
        });

        log::trace!(?app_info, "newly found AppInfo");

        Ok(app_info)
    }

    // TODO maybe extract all of the below out into a struct like AppInformation?
    async fn get_app_file_info(
        app_id: model::Id,
        tx: ProcessorTx,
        identity: model::AppIdentity,
    ) -> Result<()> {
        let file_info: FileInfo = match &identity {
            model::AppIdentity::Win32 { path } => FileInfo::from_win32(path)
                .await
                .with_context(|| "Retrieve FileInfo from Win32 path")?,
            model::AppIdentity::UWP { aumid } => FileInfo::from_uwp(aumid)
                .await
                .with_context(|| "Retrieve FileInfo from UWP aumid")?,
        };

        tx.send(Message::AppUpdate { app_id, file_info })
            .with_context(|| "Send AppUpdated message")?;

        Ok(())
    }

    fn find_or_create_app(
        window: &Window,
        process: &Process,
        db: &mut Database,
        tx: &ProcessorTx,
    ) -> Result<model::App> {
        let identity = Info::get_identity(window, process)
            .with_context(|| "Getting AppIdentity of Process")?;
        match db
            .app_by_identity(&identity)
            .with_context(|| "Find existing App by AppIdentity")?
        {
            Some(app) => Ok(app),
            None => {
                let identity = Info::get_identity(window, process)
                    .with_context(|| "Get Identity of Process and Window")?;

                let mut app = model::App {
                    id: 0,
                    name: None,
                    description: None,
                    color: Some("#EEEEEE".to_string()),
                    identity: identity.clone(),
                };
                db.insert_app(&mut app)
                    .with_context(|| "Saving App to Database")?;

                // TODO find better way to extend the lifetime of the mutable db reference
                task::spawn(Info::get_app_file_info(app.id, tx.clone(), identity));

                Ok(app)
            }
        }
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
