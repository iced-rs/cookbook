use copy_image_to_clipboard::convert_svg_to_png::*;
use copy_image_to_clipboard::copy_image_to_clipboard::*;
use iced::svg::Handle;
use iced::{button, Button, Column, Container, Element, Length, Row, Sandbox, Settings, Svg, Text};

pub fn main() -> iced::Result {
    Copier::run(Settings::default())
}

#[derive(Default)]
struct Copier {
    button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPressed,
}

impl Sandbox for Copier {
    type Message = Message;

    fn new() -> Self {
        Copier::default()
    }

    fn title(&self) -> String {
        String::from("Copier")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => {
                let svg_bytes = include_bytes!["../img/quad_hive.svg"].to_vec();
                let png = convert_svg_to_png(&svg_bytes);
                copy_png_bytes_to_mac_clipboard(&png);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let button = Button::new(&mut self.button, Text::new("Copy"))
            .padding(10)
            .on_press(Message::ButtonPressed);
        let svg_bytes = include_bytes!["../img/quad_hive.svg"].to_vec();
        let svg = Svg::new(Handle::from_memory(svg_bytes));
        let row = Row::new().push(svg).push(button);
        let content = Column::new().push(row);
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
