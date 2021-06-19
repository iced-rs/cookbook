use iced::{button, Align, Button, Element, Text, Row};
use iced_winit::{Program, Clipboard, Command};
use iced_wgpu::Renderer;

#[derive(Default)]
pub struct CounterApp {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
    reset_button: button::State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
    ResetPressed,
}

impl CounterApp {
    pub fn new() -> CounterApp {
        CounterApp {
            value: Default::default(),
            increment_button: Default::default(),
            decrement_button: Default::default(),
            reset_button: Default::default(),
        }
    }
}

impl Program for CounterApp {
    type Message = Message;

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message>{
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::ResetPressed => {
                self.value = 0;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Row::new()
            .padding(5)
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new(self.value.to_string()).size(50))
            .push(
                Button::new(&mut self.increment_button, Text::new("Increment"))
                    .on_press(Message::IncrementPressed),
            )
            .push(
                Button::new(&mut self.decrement_button, Text::new("Decrement"))
                    .on_press(Message::DecrementPressed),
            )
            .push(Button::new(&mut self.reset_button, Text::new("Reset"))
                .on_press(Message::ResetPressed))
            .into()
    }

    type Renderer = Renderer;
    type Clipboard = Clipboard;
}