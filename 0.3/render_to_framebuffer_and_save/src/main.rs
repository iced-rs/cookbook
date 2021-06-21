mod counter_app;
mod generator;
use generator::generate_png;
use iced::futures::executor::block_on;

pub fn main() {
    block_on(generate_png())
}