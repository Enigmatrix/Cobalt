use common::errors::*;
use common::settings::Settings;
use common::tracing::*;

fn main() -> Result<()> {
    let settings = Settings::from_file("appsettings.json").context("fetch settings")?;
    common::setup(&settings).context("setup common")?;
    platform::setup().context("setup platform")?;
    info!("engine is running");

    let ev = platform::objects::EventLoop::new();
    let every = platform::objects::Duration::from_millis(1_000);

    let _timer = platform::objects::Timer::new(every, every, &mut || {
        if let Some(window) = platform::objects::Window::foreground() {
            let process = platform::objects::Process::new(window.pid()?)?;
            let path = process.path()?;
            info!(title = window.title()?);
            info!(path = path);
            if process.is_uwp(Some(&path))? {
                info!(aumid = window.aumid()?);
            }
        }

        Ok(())
    });

    ev.run();

    Ok(())
}
