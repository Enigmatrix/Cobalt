use iced::*;

mod process;

use native::raw::*;
use process::*;

fn main() -> anyhow::Result<()> {
    native::setup()?;
    Monitor::run(Settings::default());
    Ok(())
}

struct Monitor {
    apps: Vec<App>,
    selected: Option<AppInfo>,

    refresh_button: button::State,
    cmdline_text: text_input::State,
}

#[derive(Debug, Clone)]
enum Message {
    CommandLineChanged(String),
    RefreshProcesses,
    Select(App),
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            apps: Monitor::get_running_window_processes(),
            selected: None,

            refresh_button: button::State::new(),
            cmdline_text: text_input::State::new(),
        }
    }

    pub fn get_running_window_processes() -> Vec<App> {
        let mut apps: Vec<App> = WindowEnumerator::new()
            .unwrap()
            .into_windows()
            .into_iter()
            .filter(|w| unsafe { winuser::IsWindowVisible(w.0) != 0 })
            .filter_map(|w| App::new_from_window(w).ok())
            .collect();
        apps.sort_by_cached_key(|x| x.pid);
        apps.dedup_by_key(|x| x.pid);
        apps
    }
}

impl Application for Monitor {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Monitor::new(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Monitor - Cobalt Experiments")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RefreshProcesses => {
                self.apps = Monitor::get_running_window_processes();
            }
            Message::CommandLineChanged(s) => {
                self.selected.as_ref().unwrap().write_cmd(s);
                self.selected = Some(AppInfo::new(self.selected.as_ref().unwrap().process.pid().unwrap()));
            }
            Message::Select(app) => {
                self.selected = Some(AppInfo::new(app.pid));
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let mut processes = Column::new()
            .push(
                Row::new().push(Text::new("Processes").size(32)).push(
                    Button::new(&mut self.refresh_button, Text::new("REFRESH"))
                        .on_press(Message::RefreshProcesses),
                ),
            )
            .max_width(400);

        for app in &mut self.apps {
            let mut buffer = Vec::with_capacity(4096);
            // let bstyle = button::Style { background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.0))), ..button::Style::default() };
            let app2 = app.clone();
            app.icon
                .write_to(&mut buffer, ::image::ImageOutputFormat::Png)
                .unwrap();
            processes = processes.push(
                Button::new(
                    &mut app.selected_state,
                    Row::new()
                        .push(
                            Container::new(
                                Image::new(iced::widget::image::Handle::from_memory(buffer))
                                    .width(Length::Units(32))
                                    .height(Length::Units(32)),
                            )
                            .padding(10),
                        )
                        .push(
                            Column::new()
                                .push(Text::new(app.name.clone()).size(20))
                                .push(Text::new(app.desc.clone()).size(16))
                                .push(Text::new(app.path.clone()).size(10)),
                        ),
                )
                .on_press(Message::Select(app2)),
            );
        }

        let mut info = Column::new().push(Text::new("Info").size(32));
        if let Some(app_info) = &self.selected {
            let (img2, cmd) = app_info.path_and_cmd();
            info = info
                .push(TextInput::new(
                    &mut self.cmdline_text,
                    "new commandline",
                    cmd.as_str(),
                    |x| Message::CommandLineChanged(x),
                ))
                .push(Text::new(format!("CommandLine:\t{}", cmd)).size(14))
                .push(
                    Text::new(format!(
                        "QueryFullProcessImageName:\t{}",
                        app_info.process.path().unwrap()
                    ))
                    .size(14),
                )
                .push(Text::new(format!("ImagePathName:\t{}", img2)).size(14));
        }

        Row::new().push(processes).push(info).padding(20).into()
    }
}
