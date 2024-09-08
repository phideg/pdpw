use iced::keyboard::key;
use iced::widget::{
    self, button, column, container, horizontal_space, row, text, text_editor, text_input,
};
use iced::{event, keyboard, Event};
use iced::{Element, Length, Subscription, Task};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::store::{self, load_pdpw_file, store_pdpw_file};

#[derive(Debug)]
enum ModalState {
    Search,
    Pin,
    None,
}
pub(crate) struct Editor {
    content: text_editor::Content,
    error: Option<String>,
    is_dirty: bool,
    is_loading: bool,
    modal: ModalState,
    pdpw_file: PathBuf,
    pin: String,
    search_string: String,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ActionPerformed(text_editor::Action),
    ContentLoaded(Result<Arc<String>, Error>),
    Event(Event),
    FileSaved(Result<PathBuf, Error>),
    HideModal,
    LoadPdwpFile,
    PinInput(String),
    Search,
    SearchString(String),
    SetPdpwPath(PathBuf),
}

impl Editor {
    pub(crate) fn new(pdpw_file_path: &str) -> (Self, Task<Message>) {
        (Self {
            content: text_editor::Content::new(),
            error: None,
            is_dirty: false,
            is_loading: true,
            modal: ModalState::Pin,
            pdpw_file: PathBuf::new(),
            pin: String::new(),
            search_string: String::new(),
        },
        Task::perform(
            set_pdpw_path(PathBuf::from(pdpw_file_path)),
            Message::SetPdpwPath,
        ))
    }

    fn hide_modal(&mut self) {
        self.modal = ModalState::None;
        self.error = None;
        self.search_string.clear();
    }

    fn run_save_file(&mut self) -> Task<Message> {
        if self.is_loading {
            Task::none()
        } else {
            self.is_loading = true;
            self.is_dirty = false;
            Task::perform(
                save_file(
                    self.pdpw_file.clone(),
                    self.pin.clone(),
                    self.content.text(),
                ),
                Message::FileSaved,
            )
        }
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Task::none()
            }
            Message::ContentLoaded(result) => {
                match result {
                    Ok(contents) => {
                        self.hide_modal();
                        self.content = text_editor::Content::with_text(&contents)
                    }
                    Err(error) => {
                        dbg!(&error);
                        self.error = Some(format!("{error:?}"))
                    }
                }
                Task::none()
            }
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(c),
                    modifiers,
                    ..
                }) if modifiers.command() => match c.as_str() {
                    "s" => self.run_save_file(),
                    "f" => {
                        self.modal = ModalState::Search;
                        widget::focus_next()
                    }
                    _ => Task::none(),
                },
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) => {
                    self.hide_modal();
                    Task::none()
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Enter),
                    ..
                }) => {
                    self.hide_modal();
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::LoadPdwpFile => {
                self.is_loading = false;
                if self.pin.is_empty() {
                    Task::none()
                } else {
                    Task::perform(
                        load_content(self.pdpw_file.clone(), self.pin.clone()),
                        Message::ContentLoaded,
                    )
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;
                match result {
                    Ok(path) => self.pdpw_file = path,
                    Err(e) => {
                        self.is_dirty = true;
                        self.error = Some(format!("{e:?}"))
                    }
                }
                Task::none()
            }
            Message::HideModal => {
                self.modal = ModalState::None;
                Task::none()
            }
            Message::PinInput(pin) => {
                self.pin = pin;
                Task::none()
            }
            Message::Search => {
                // simple exact search
                let text = self.content.text();
                if let Some(found) = text.find(self.search_string.as_str()) {
                    // update the cursor
                    dbg!(found);
                    self.content.perform(text_editor::Action::SelectLine);
                }
                Task::none()
            }
            Message::SearchString(search_string) => {
                self.search_string = search_string;
                Task::none()
            }
            Message::SetPdpwPath(pdpw_file) => {
                self.pdpw_file = pdpw_file;
                widget::focus_next()
            }
            _ => todo!(),
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    pub(crate) fn view(&self) -> Element<Message> {
        let mut info = if let Some(err_msg) = self.error.as_deref() {
            err_msg.to_string()
        } else {
            self.pdpw_file.display().to_string()
        };
        info.push_str(if self.is_dirty { " [dirty]" } else { " [OK]" });
        let status = row![
            text(if info.len() > 60 {
                format!("...{}", &info[info.len() - 40..])
            } else {
                info
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        let content = column![
            text_editor(&self.content)
                .height(Length::Fill)
                .on_action(Message::ActionPerformed),
            status,
        ]
        .spacing(10)
        .padding(10);

        match self.modal {
            ModalState::None => content.into(),
            ModalState::Pin => {
                let popup = container(
                    column![
                        text("Enter your master password").size(24),
                        column![text_input("", &self.pin,)
                            .on_input(Message::PinInput)
                            .on_submit(Message::LoadPdwpFile)
                            .padding(5),]
                        .spacing(5),
                        button(text("OK")).on_press(Message::LoadPdwpFile),
                    ]
                    .spacing(20),
                )
                .width(300)
                .padding(10)
                .style(container::rounded_box);
                crate::modal::modal(content, popup, Message::HideModal)
            }
            ModalState::Search => {
                let popup = container(
                    column![
                        text("Search pattern:").size(24),
                        text_input("", &self.search_string,)
                            .on_input(Message::SearchString)
                            .on_submit(Message::Search)
                            .padding(5),
                        button(text("Search")).on_press(Message::HideModal),
                    ]
                    .spacing(20),
                )
                .width(300)
                .padding(10)
                .style(container::rounded_box);
                crate::modal::modal(content, popup, Message::HideModal)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    LoadError(String),
    SaveError(String),
    Unexpected(String),
}

async fn load_content(path: PathBuf, pin: String) -> Result<Arc<String>, Error> {
    let contents = load_pdpw_file(path.as_path(), &pin)
        .await
        .map_err(|e| Error::LoadError(format!("Couldn't load *pdpw file from {path:?}: [{e}]")))?;
    Ok(Arc::new(contents))
}

async fn set_pdpw_path(path: PathBuf) -> PathBuf {
    path
}

async fn save_file(path: PathBuf, pin: String, contents: String) -> Result<PathBuf, Error> {
    store_pdpw_file(&path, pin.as_str(), contents.as_str())
        .await
        .map_err(|e| Error::SaveError(format!("{e}")))?;
    Ok(path)
}
