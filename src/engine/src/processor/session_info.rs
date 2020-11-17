use super::*;
use crate::data::db::Database;
use crate::data::model;

#[derive(Debug)]
pub struct SessionInfo {
    pub closed_watcher: window_closed::Watcher,
    pub session: model::Session,
    pub pid: ProcessId,
}

impl SessionInfo {
    pub fn get<'a>(
        window: &'a Window,
        msger: &'a Messenger,
        db: &'a mut Database,
        sessions: &'a mut SessionCache,
        apps: &'a mut AppCache,
    ) -> Result<(&'a mut SessionInfo, &'a mut AppInfo)> {
        Ok(match sessions.entry(window.clone()) {
            Occupied(occ) => {
                let session_info = occ.into_mut();
                let app_info = apps.get_mut(&session_info.pid).unwrap(); // if the session exists, the app for it exists.
                (session_info, app_info)
            }
            Vacant(vac) => {
                let (session_info, app_info) = SessionInfo::new(window, msger, db, apps)
                    .with_context(|| "Create new SessionInfo")?;
                let session_info = vac.insert(session_info);
                (session_info, app_info)
            }
        })
    }

    fn new<'a>(
        window: &'a Window,
        msger: &'a Messenger,
        db: &'a mut Database,
        apps: &'a mut AppCache,
    ) -> Result<(SessionInfo, &'a mut AppInfo)> {
        let (pid, _) = window.pid_tid()?;

        let _msger = msger.clone();
        let closed_watcher = window_closed::Watcher::new(window, move |window| {
            _msger
                .send(Message::WindowClosed { window })
                .with_context(|| "Send WindowClosed message")
        })
        .with_context(|| "Create Window closed watcher for new SessionInfo")?;

        let app_info = AppInfo::get(window, msger, db, apps).with_context(|| "Getting AppInfo")?;

        // always create a new session, no need to access db.
        let session = db.insert_session(model::Session {
            id: 0,
            app_id: app_info.app.id,
            arguments: app_info.arguments.clone(),
            title: window
                .title()
                .with_context(|| "Get title of Window for Session")?,
        })?;

        Ok((
            SessionInfo {
                closed_watcher,
                session,
                pid,
            },
            app_info,
        ))
    }
}
