use iced::{Scrollable, scrollable, Application, Container, Text, Element, Column, Command, Settings, Length, HorizontalAlignment, Clipboard, Font, Row, Button, Space, button, TextInput, text_input};
use std::path::Path;
use std::fs;
use std::cmp::min;
use lipsum::{lipsum, lipsum_title};
use std::time::{SystemTime, UNIX_EPOCH};

mod logger;
use logger::search_files;

pub const LOGS: &str = "./logs";
const LOG_MAX: usize = 100;

enum App {
    Loading,
    Loaded(State)
}

struct State {
    scroll: scrollable::State,
    logs: Vec<Log>,
    unsearched_files: Vec<String>,
    search_bars: Vec<SearchBar>,
    create_button: button::State,
    num_to_create: u8,
    search_start: Option<SystemTime>,
    speed_text: String,
}

#[derive(Debug, Clone)]
struct LoadState {
    // fill with async loaded things
}

#[derive(Debug, Clone)]
enum LoadError {
    // Placeholder for if async load fails
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<LoadState, LoadError>),
    Log(usize, LogMessage),
    SearchChanged(usize, SearchBarMessage),
    AddLog((Vec<String>, Option<Log>)),
    MoarFiles,
    MoarFiled(()),
    GotSpeed(Option<(u128, u128, u64)>)
}

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App::Loading,
            Command::perform(LoadState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Application Boiler Plate")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            App::Loading => {
                match message {
                    // when async completes this is called & converts LoadState to State, which is then
                    // displayed using view()
                    // `_LoadState' is not used here, but will likely be needed in your program.
                    // remove the underscore and then the data in LoadState can be moved over to 
                    Message::Loaded(Ok(_load_state)) => {
                        *self = App::Loaded(State{
                            scroll: scrollable::State::new(),
                            logs: Vec::new(),
                            search_bars: vec![SearchBar::new(0)],
                            unsearched_files: Vec::new(),
                            create_button: button::State::new(),
                            num_to_create: 0,
                            search_start: None,
                            speed_text: "Create files and use the search below to feel the speed of Rust + Iced.".to_string(),
                        })
                    },
                    _ => ()
                }
                Command::none()
            }
            App::Loaded(state) => {
                match message {
                    Message::AddLog((vals, log)) => {
                        if state.logs.len() <= LOG_MAX
                            && vals
                                .iter()
                                .zip(state.search_bars.iter().fold(
                                    Vec::with_capacity(state.search_bars.len()),
                                    |mut v, bar| {
                                        v.push(&bar.value);
                                        v
                                    },
                                ))
                                .all(|(a, b)| a == &b.to_lowercase())
                        {
                            if let Some(log) = log {
                                state.logs.push(log);
                            }
                            if state.unsearched_files.len() > 0 {
                                Command::perform(
                                    search_files(vals, state.unsearched_files.remove(0)),
                                    Message::AddLog,
                                )
                            } else {
                            if let Some(start_time) = state.search_start {
                                Command::perform(calc_speed(0, start_time), Message::GotSpeed)
                            } else {Command::none()}
                            }
                        } else {
                            let length = state.unsearched_files.len();
                            state.unsearched_files.clear();
                            if let Some(start_time) = state.search_start {
                                Command::perform(calc_speed(length, start_time), Message::GotSpeed)
                            } else {Command::none()}
                        }
                    }
                    Message::SearchChanged(i, SearchBarMessage::InputChanged(val)) => {
                        // run search as multithreaded Commands to speed up search
                        // update bar and add new if necessary
                        state.search_start = Some(SystemTime::now());
                        state.search_bars[i].value = val.clone();
                        if state.search_bars.len() - 1 == i {
                            state.search_bars.push(SearchBar::new(i + 1));
                        }
                        // remove empty search bars
                        if val == "".to_string() {
                            if i == 0 {
                                state.update_logs();
                                return Command::none();
                            } else {
                                state.search_bars.remove(i);
                                for i in 0..state.search_bars.len() {
                                    state.search_bars[i].num = i;
                                    state.search_bars[i].message = if i == 0 {
                                        "Search".to_string()
                                    } else {
                                        format!("Search term {} (Optional)", i + 1)
                                    };
                                }
                            }
                        }
                        if state.search_bars.len() == 1 && state.search_bars[0].value == "".to_string() {
                            state.update_logs();
                            Command::none()
                        } else {
                            state.logs = Vec::with_capacity(LOG_MAX);
                            state.unsearched_files = if let Ok(files) = fs::read_dir(Path::new(LOGS)) {
                                files.fold(
                                    Vec::with_capacity(LOG_MAX),
                                    |mut v, file| {
                                        v.push(file.unwrap().file_name().to_string_lossy().to_string());
                                        v
                                    },
                                )
                            } else {Vec::new()};
                            // Note: limit to 15 active search threads as limit on windows
                            Command::batch((0..min(15, state.unsearched_files.len())).into_iter().fold(
                                Vec::with_capacity(15),
                                |mut v, _i| {
                                    v.push(Command::perform(
                                        search_files(
                                            state.search_bars.iter().fold(
                                                Vec::with_capacity(state.search_bars.len()),
                                                |mut v, bar| {
                                                    v.push(bar.value.to_lowercase().clone());
                                                    v
                                                },
                                            ),
                                            state.unsearched_files.remove(0),
                                        ),
                                        Message::AddLog,
                                    ));
                                    v
                                },
                            ))
                        }
                    }
                    Message::Log(i, msg) => {
                        state.logs[i].update(msg);
                        Command::none()
                    }
                    Message::MoarFiles => {
                        state.num_to_create = 100;
                        Command::batch
                            ((0..100)
                                .into_iter()
                                .fold(Vec::with_capacity(100), |mut vec, _i| {
                                vec.push(Command::perform(
                                        create_file(), Message::MoarFiled
                                    ));
                                vec
                                }))
                    }
                    Message::MoarFiled(_) => {
                        state.num_to_create -= 1;
                        Command::none()
                    }
                    Message::GotSpeed(result) => {
                        match result {
                            Some(speed) if state.unsearched_files.len() == 0 => {
                                state.speed_text = format!("{}ms, ~{}Mb/s ({} files total)", speed.0, speed.1, speed.2);
                            }
                            _ => ()
                        }
                        Command::none()
                    }
                    _ => Command::none()
                }
            }

        }
    }
    fn view(&mut self) -> Element<Message> {
        match self {
            App::Loading => loading_message(),
            App::Loaded(State {
                    // list state variables to be accessable 
                    scroll,
                    logs,
                    unsearched_files,
                    search_bars,
                    create_button,
                    num_to_create,
                    speed_text,
                    ..
            }) => {
        let logs = logs.iter_mut().take(LOG_MAX);
        let logs_count = logs.len();
        let speed_row = Row::new().spacing(50)
                .push(if *num_to_create == 0 {
                        Button::new(create_button, Text::new("Create 100 files (~70kb)")).on_press(Message::MoarFiles)
                    } else {
                        Button::new(create_button, Text::new("createing files"))
                    })
                .push(Text::new(if unsearched_files.len() != 0 {"Caluclating Speed..."} else {speed_text}));
        let page: Element<_> = Column::new()
            .push(
                search_bars
                    .iter_mut()
                    .enumerate()
                    .fold(Column::new(), |col, (i, bar)| {
                        col.push(
                            bar.view()
                                .map(move |msg| Message::SearchChanged(i, msg)),
                        )
                    }),
            )
            .push(logs.enumerate().fold(Column::new(), |col, (i, log)| {
                col.push(log.view().map(move |msg| Message::Log(i, msg)))
            }))
            .push(if logs_count == LOG_MAX {
                Row::with_children(vec![Text::new(format!(
                    "Showing first {}. Use search to narrow down results.",
                    LOG_MAX
                ))
                .width(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
                .into()])
                .spacing(10)
            } else {
                Row::with_children(vec![Text::new(if unsearched_files.len() > 0 {
                    "Searching . . ."
                } else {
                    "Showing all results."
                })
                .width(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
                .into()])
            })
            .into();
                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(speed_row)
                    .push(page);

                Scrollable::new(scroll)
                    .padding(40)
                    .push(
                        Container::new(content)
                            .width(Length::Fill)
                            .center_x()
                    )
                    .into()
            }
        }
    }

}

impl State {
    pub fn update_logs(&mut self) {
        self.search_bars = vec![SearchBar::new(0)];
        self.logs = if let Ok(files) = fs::read_dir(Path::new(LOGS)) {
            files.map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .take(LOG_MAX)
            .map(|file| Log::new(file))
            .collect()
        } else {Vec::new()}
    }
}

impl LoadState {
    // this is the function that is called to load data
    async fn load() -> Result<LoadState, LoadError> {
        Ok(LoadState{})
    }
}

// what is displayed while waiting for the `async fn load()`
fn loading_message<'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

// This is the logic for the search bars. Their state is stored in the main state.
// It is best to keep your application's logic as 'flat' as possible, so if
// you only have one search bar just keep the whole thing as part of the main logic.
// These are only done this way to allow for an unknown number of search bars at run time.
#[derive(Clone, Debug)]
pub struct SearchBar {
    title: String,
    message: String,
    num: usize,
    value: String,
    state: text_input::State,
}

#[derive(Clone, Debug)]
pub enum SearchBarMessage {
    InputChanged(String),
}
impl SearchBar {
    fn new(num: usize) -> Self {
        SearchBar {
            title: if num == 0 {
                "Contains:".to_string()
            } else {
                "and:".to_string()
            },
            message: if num == 0 {
                "Search".to_string()
            } else {
                format!("Search term {} (Optional)", num + 1)
            },
            num,
            value: String::new(),
            state: text_input::State::new(),
        }
    }

    fn view(&mut self) -> Element<'_, SearchBarMessage> {
        Row::new()
            .push((0..self.num).fold(Row::new(), |r, _i| {
                r.push(Space::with_width(Length::Units(30)))
            }))
            .push(
                Row::new()
                    .push(
                        Text::new(&self.title)
                            .size(20)
                            .width(Length::Units(80))
                            .horizontal_alignment(HorizontalAlignment::Right),
                    )
                    .padding(10),
            )
            .push(
                TextInput::new(
                    &mut self.state,
                    &self.message[..],
                    &self.value,
                    SearchBarMessage::InputChanged,
                )
                .padding(10),
            )
            .into()
    }
}

// These are the drop down boxes that contain the text that matches.
// These in general just show the concept of a drop down in iced.
// Click button -> Toggle flag -> show more data. I'm sure that
// there are many optimizations for grabbing and storing the text
// shown in the drop down, just for simplicity the text in each 'Log'
// drop down is lazilly grabbed when opened.
#[derive(Clone, Debug)]
pub struct Log {
    title: String,
    content: String,
    opened: bool,
    toggle_view_btn: button::State,
}

#[derive(Debug, Clone)]
pub enum LogMessage {
    ToggleView,
}
impl Log {
    pub fn new(title: String) -> Self {
        Log {
            title,
            content: "".to_string(), // leave empty until opened
            opened: false,
            toggle_view_btn: button::State::new(),
        }
    }

    fn update(&mut self, message: LogMessage) {
        match message {
            LogMessage::ToggleView => {
                if self.opened {
                    self.opened = false;
                } else {
                    self.content =
                        fs::read_to_string(Path::new(&format!("{}/{}", LOGS, &self.title)))
                            .unwrap_or(format!("Error: Unable to read file {}!", &self.title));
                    self.opened = true;
                }
            }
        }
    }

    fn view(&mut self) -> Element<'_, LogMessage> {
        match self.opened {
            true => Column::new()
                .push(
                    Button::new(
                        &mut self.toggle_view_btn,
                        Row::new()
                            .push(down_icon())
                            .push(Text::new(&self.title)),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .on_press(LogMessage::ToggleView),
                )
                .push(
                    Row::new()
                        .push(Text::new(&self.content).width(Length::Fill))
                        .padding(20),
                )
                .into(),
            false => Column::new()
                .spacing(5)
                .push(
                    Button::new(
                        &mut self.toggle_view_btn,
                        Row::new()
                            .push(right_icon())
                            .push(Text::new(&self.title)),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .on_press(LogMessage::ToggleView),
                )
                .into(),
        }
    }
}

// some more simple helper functions

// Fonts
const ICONS_FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/Icons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS_FONT)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}
pub fn right_icon() -> Text {
    icon('\u{E803}')
}
pub fn down_icon() -> Text {
    icon('\u{E802}')
}

async fn create_file() -> () {
    let title = lipsum_title();
    let _ = fs::write(format!("{}/{}.txt", LOGS, title), format!("{}.txt\n{}", title, lipsum(100)));
}

// returns (time (ms), Mb/s)
    async fn calc_speed(remaining: usize, start_time: SystemTime) -> Option<(u128, u128, u64)> {
    let stop_time = SystemTime::now();
    let mut files_len = 0;
    let total_time = (stop_time.duration_since(UNIX_EPOCH).expect("Time went backwards") - start_time.duration_since(UNIX_EPOCH).expect("Time went backwards")).as_millis();
    if let Ok(files) = fs::read_dir(Path::new(LOGS)) {
        Some(( total_time,
        ((files.into_iter().fold(0, |mut total_bytes, file| {
            files_len += 1;
            total_bytes += fs::metadata(file.unwrap().path()).unwrap().len();
            total_bytes
        })/files_len) as u128 * (files_len - remaining as u64) as u128/total_time) / 1048
        , files_len))
    } else {None}
}