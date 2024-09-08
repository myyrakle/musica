use iced::widget::{button, column, text};
use iced::{Alignment, Element, Sandbox, Settings, Size};

pub fn main() -> iced::Result {
    let mut setting = Settings::default();

    setting.window.size = Size::new(300.0, 600.0);

    Player::run(setting)
}

struct Player {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Player {
    type Message = Message;

    fn new() -> Self {
        Self { value: 0 }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            button("Increment").on_press(Message::IncrementPressed),
            text(self.value).size(50),
            button("Decrement").on_press(Message::DecrementPressed)
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}
