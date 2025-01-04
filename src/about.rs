use iced::widget::{button, center, column, text};
use iced::{window, Font};
use iced::{Center, Element, Task};

#[derive(Default)]
pub(crate) struct MsgPopup {
    msg: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Message {
    Confirm,
}

impl MsgPopup {
    pub(crate) fn new(msg: String) -> (Self, Task<Message>) {
        (Self { msg }, Task::none())
    }

    pub(crate) fn update(&mut self, _message: Message) -> Task<Message> {
        window::get_latest().and_then(window::close)
    }

    pub(crate) fn view(&self) -> Element<Message> {
        let content = column![
            text(&self.msg).font(Font::MONOSPACE),
            button("Yes, exit now")
                .padding([10, 20])
                .on_press(Message::Confirm),
        ]
        .spacing(10)
        .align_x(Center);

        center(content).padding(20).into()
    }
}
