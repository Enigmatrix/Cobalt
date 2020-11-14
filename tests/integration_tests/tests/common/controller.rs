use native::wrappers::Window as NativeWindow;
use std::path::Path;

/*
pub struct Controller {
    apps: Vec<App>
}

impl Controller {
    pub fn new() -> Controller { Controller { apps: Vec::new() } }
    pub fn spawn_app<'a, P: AsRef<Path>>(&mut self, path: P) -> AppRef<'a> {
        let app = App::start(path).unwrap();
        let idx = self.apps.push(value)
        AppRef { controller: self, index: idx }
     }
}

pub struct AppRef<'a> {
    controller: &'a Controller,
    index: usize
}

pub struct App {
}

impl App {
    pub fn start<P: AsRef<Path>>(path: P) -> Result<App, String> {
        Ok(App {})
    }

    pub fn main_window(&self) -> Option<Window> {
        None
    }
}

pub struct Window {
    inner: NativeWindow
}

impl Window {
    pub fn make_foreground() -> Result<(), String> {
        Ok(())
    }
}*/
