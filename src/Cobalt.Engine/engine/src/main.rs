use common::errors::*;
use common::settings::Settings;
use common::tracing::*;

fn main() -> Result<()> {
    let settings = Settings::from_file("appsettings.json").context("fetch settings")?;
    common::setup(&settings).context("setup common")?;
    info!("engine is running");

    loop {
        if let Some(window) = platform::objects::Window::foreground() {
            let process = platform::objects::Process::new(window.pid()?)?;
            let path = process.path()?;
            info!(title = window.title()?);
            info!(path = path);
            if process.is_uwp(Some(&path))? {
                info!(aumid = window.aumid()?);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
