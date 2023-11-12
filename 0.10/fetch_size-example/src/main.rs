use iced::Application;

struct App {
    button_text: String,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ScreenSize(u32, u32),
    GetScreenSize,
}

impl iced::Application for App {
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;

    fn title(&self) -> String {
        String::from("fetch_size example")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::GetScreenSize => {
                // return the command, that will produce the ScreenSize Message, generated by fetch_size
                return iced::window::fetch_size(|size| {
                    Message::ScreenSize(size.width, size.height)
                });
            }
            // react to the Message produced by the command returned above
            Message::ScreenSize(width, height) => {
                self.button_text = format!("{}x{}", width, height);
            }
        }
        // return empty command because it is required by the function signature
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        // draw a button with the window size as text and producing the GetScreenSize Message
        iced::widget::button(&*self.button_text)
            .on_press(Message::GetScreenSize)
            .into()
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                button_text: String::from("?x?"),
            },
            iced::Command::none(),
        )
    }
}

fn main() {
    App::run(iced::Settings::default()).unwrap();
}
