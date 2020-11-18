use super::*;
use crate::data::db::Database;
use crate::data::model;

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

pub struct Info;

impl Info {
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
                let (session_info, app_info) = Info::new_session_info(window, msger, db, apps)
                    .with_context(|| "Create new SessionInfo")?;
                let session_info = vac.insert(session_info);
                (session_info, app_info)
            }
        })
    }

    fn new_session_info<'a>(
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

        let app_info =
            Info::get_app_info(window, msger, db, apps).with_context(|| "Getting AppInfo")?;

        // always create a new session, no need to access db.
        let mut session = model::Session {
            id: 0,
            app_id: app_info.app.id,
            arguments: app_info.arguments.clone(),
            title: window
                .title()
                .with_context(|| "Get title of Window for Session")?,
        };
        db.insert_session(&mut session)
            .with_context(|| "Saving Session to Database")?;

        Ok((
            SessionInfo {
                closed_watcher,
                session,
                pid,
            },
            app_info,
        ))
    }

    pub fn get_app_info<'a>(
        window: &Window,
        msger: &Messenger,
        db: &mut Database,
        apps: &'a mut AppCache,
    ) -> Result<&'a mut AppInfo> {
        let (pid, _) = window.pid_tid()?;
        Ok(match apps.entry(pid) {
            Occupied(occ) => occ.into_mut(),
            Vacant(vac) => vac.insert(
                Info::new_app_info(
                    window,
                    msger,
                    Process::new(pid, ProcessOptions::default())?,
                    db,
                )
                .with_context(|| "Create new AppInfo")?,
            ),
        })
    }

    fn new_app_info(
        window: &Window,
        msger: &Messenger,
        process: Process,
        db: &mut Database,
    ) -> Result<AppInfo> {
        let arguments = process.cmd().ok();
        let app = Info::find_or_create_app(window, &process, db)
            .with_context(|| "Find/creating App for process")?;

        let exited_watcher = process_exit::Watcher::new(&process, move |pid| {
            msger
                .send(Message::ProcessExit { pid })
                .with_context(|| "Send ProcessExit message")
        })
        .with_context(|| "Creating process exit watcher for AppInfo")?;

        Ok(AppInfo {
            exited_watcher,
            process,
            arguments,
            app,
        })
    }

    // TODO maybe extract this & get_identity out?
    fn find_or_create_app(
        window: &Window,
        process: &Process,
        db: &mut Database,
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
                let mut app = match &identity {
                    model::AppIdentity::Win32 { path } => {
                        let file = FileInfo::from_classic_app(path)
                            .with_context(|| "Retreive file info of Win32 executable")?;
                        model::App {
                            id: 0,
                            name: file.name,
                            description: file.description,
                            color: Info::find_contrasting_color(&file.icon)
                                .with_context(|| "Finding contrasting color of image")?,
                            identity,
                        }
                    }
                    model::AppIdentity::UWP { aumid } => todo!("Construct UWP App for {}", aumid),
                };
                db.insert_app(&mut app)
                    .with_context(|| "Saving App to Database")?;
                Ok(app)
            }
        }
    }

    fn find_contrasting_color(image: &image::DynamicImage) -> Result<String> {
        let colfmt = match image.color() {
            image::ColorType::Rgb8 => color_thief::ColorFormat::Rgb,
            image::ColorType::Rgba8 => color_thief::ColorFormat::Rgba,
            _ => unreachable!(),
        };
        let colors = color_thief::get_palette(&image.to_bytes(), colfmt, 1, 3)?;
        let dominant = colors[0];
        let mut contrasting = hsl::HSL::from_rgb(&[dominant.r, dominant.g, dominant.b]);
        contrasting.h = (contrasting.h + 180.0) % 360.0;
        let (r, g, b) = contrasting.to_rgb();
        Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
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
