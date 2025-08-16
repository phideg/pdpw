use iced::keyboard::key;
use iced::widget::{
    self, button, checkbox, column, container, horizontal_space, row, text, text_editor, text_input,
};
use iced::{Element, Length, Subscription};
use iced::{Event, Task, event, keyboard};

use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use crate::VERSION;
use crate::store::{load_pdpw_file, store_pdpw_file};

#[derive(Debug, PartialEq)]
enum ModalState {
    Search,
    Pin,
    UpdatePin,
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
    old_pin: String,
    new_pin: String,
    search_string: String,
    case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ActionPerformed(text_editor::Action),
    ContentLoaded(Result<Arc<String>, Error>),
    Event(Event),
    FileSaved(Result<PathBuf, Error>),
    HideModal,
    LoadPdpwFile,
    NewPinInput(String),
    NoHideModal,
    OldPinInput(String),
    OpenSearch,
    OpenSetPin,
    PinInput(String),
    SavePdpwFile,
    Search,
    SearchString(String),
    SetNewPassword,
    SetPdpwPath(PathBuf),
    ToggleCaseSensitive(bool),
}

impl Editor {
    pub(crate) fn new(pdpw_file_path: &str) -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                error: None,
                is_dirty: false,
                is_loading: true,
                modal: ModalState::Pin,
                pdpw_file: PathBuf::new(),
                pin: String::new(),
                old_pin: String::new(),
                new_pin: String::new(),
                search_string: String::new(),
                case_sensitive: false,
            },
            Task::perform(
                set_pdpw_path(PathBuf::from(pdpw_file_path)),
                Message::SetPdpwPath,
            ),
        )
    }

    fn hide_modal(&mut self) {
        self.modal = ModalState::None;
        self.error = None;
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
                if self.modal == ModalState::None {
                    self.is_dirty = self.is_dirty || action.is_edit();
                    self.content.perform(action);
                }
                Task::none()
            }
            Message::ContentLoaded(result) => {
                match result {
                    Ok(contents) => {
                        self.hide_modal();
                        self.content = text_editor::Content::with_text(&contents)
                    }
                    Err(error) => self.error = Some(format!("{error:?}")),
                }
                widget::focus_next()
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
                }) if modifiers.command() && self.modal != ModalState::Pin => match c.as_str() {
                    "s" => self.run_save_file(),
                    "f" => {
                        self.modal = ModalState::Search;
                        text_input::focus("search-input")
                    }
                    _ => Task::none(),
                },
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::F3),
                    ..
                }) => {
                    if self.modal == ModalState::None {
                        self.execute_search(true);
                    }
                    Task::none()
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) => {
                    if self.modal != ModalState::Pin {
                        self.hide_modal();
                    }
                    Task::none()
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Enter),
                    ..
                }) => {
                    if self.modal != ModalState::Pin {
                        self.hide_modal();
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::LoadPdpwFile => {
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
            Message::SavePdpwFile => self.run_save_file(),
            Message::SetNewPassword => {
                if self.pin != self.old_pin {
                    self.error = Some("Old password does not match!".into());
                    Task::none()
                } else {
                    self.pin = self.new_pin.clone();
                    self.hide_modal();
                    self.run_save_file()
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
            Message::NoHideModal => {
                self.error = Some("Please finish the dialog first!".into());
                Task::none()
            }
            Message::PinInput(pin) => {
                self.pin = pin;
                Task::none()
            }
            Message::OldPinInput(pin) => {
                self.old_pin = pin;
                Task::none()
            }
            Message::NewPinInput(pin) => {
                self.new_pin = pin;
                Task::none()
            }
            Message::OpenSearch => {
                self.modal = ModalState::Search;
                text_input::focus("search-input")
            }
            Message::OpenSetPin => {
                self.modal = ModalState::UpdatePin;
                text_input::focus("old-pin-input")
            }
            Message::Search => {
                self.hide_modal();
                self.execute_search(false);
                widget::focus_next()
            }
            Message::SearchString(search_string) => {
                self.search_string = search_string;
                Task::none()
            }
            Message::ToggleCaseSensitive(is_checked) => {
                self.case_sensitive = is_checked;
                Task::none()
            }
            Message::SetPdpwPath(pdpw_file) => {
                self.pdpw_file = pdpw_file;
                text_input::focus("pin-input")
            }
        }
    }

    fn execute_search(&mut self, skip_current: bool) {
        if self.search_string.is_empty() {
            self.error = Some("Search string cannot be empty!".into());
            self.modal = ModalState::Search;
            return;
        }
        // simple exact search
        let (text, search_string) = if self.case_sensitive {
            (self.content.text(), self.search_string.clone())
        } else {
            (
                self.content.text().to_lowercase(),
                self.search_string.to_lowercase(),
            )
        };
        let (cursor_line, cursor_offset) = self.content.cursor_position();
        for (line_number, line) in text.lines().enumerate() {
            if line_number >= cursor_line
                && let Some(mut offset) = line.find(search_string.as_str())
            {
                if line_number == cursor_line
                    && (offset < cursor_offset || (offset == cursor_offset && skip_current))
                {
                    let updated_search_offset = cursor_offset + 1;
                    if updated_search_offset < line.len()
                        && let Some(delta_offset) =
                            line[updated_search_offset..].find(search_string.as_str())
                    {
                        offset = updated_search_offset + delta_offset;
                    } else {
                        continue;
                    }
                }
                // move the cursor to the right line
                self.content.perform(text_editor::Action::Move(
                    text_editor::Motion::DocumentStart,
                ));
                for _ in 0..line_number {
                    self.content
                        .perform(text_editor::Action::Move(text_editor::Motion::Down));
                }
                // move the cursor to the right offset
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::Home));
                for _ in 0..offset {
                    self.content
                        .perform(text_editor::Action::Move(text_editor::Motion::Right));
                }
                // select the search string
                for _ in search_string.chars() {
                    self.content
                        .perform(text_editor::Action::Select(text_editor::Motion::Right));
                }
                break;
            }
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    pub(crate) fn view(&'_ self) -> Element<'_, Message> {
        let header = row![
            button(text("Save")).on_press(Message::SavePdpwFile),
            button(text("Search")).on_press(Message::OpenSearch),
            button(text("Set Pin")).on_press(Message::OpenSetPin),
            horizontal_space(),
            text(format!("v{VERSION}")),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

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
            header,
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
                        column![
                            text_input("", &self.pin,)
                                .id("pin-input")
                                .secure(true)
                                .on_input(Message::PinInput)
                                .on_submit(Message::LoadPdpwFile)
                                .padding(5),
                        ]
                        .spacing(5),
                        button(text("OK")).on_press(Message::LoadPdpwFile),
                    ]
                    .spacing(20),
                )
                .width(300)
                .padding(10)
                .style(container::rounded_box);
                crate::modal::modal(content, popup, Message::NoHideModal)
            }
            ModalState::UpdatePin => {
                let popup = container(
                    column![
                        text("Old password").size(24),
                        column![
                            text_input("", &self.old_pin,)
                                .id("old-pin-input")
                                .secure(true)
                                .on_input(Message::OldPinInput)
                                .padding(5),
                        ]
                        .spacing(5),
                        text("New password").size(24),
                        column![
                            text_input("", &self.new_pin)
                                .id("new-pin-input")
                                .secure(true)
                                .on_input(Message::NewPinInput)
                                .on_submit(Message::SetNewPassword)
                                .padding(5),
                        ]
                        .spacing(5),
                        button(text("OK")).on_press(Message::SetNewPassword),
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
                            .id("search-input")
                            .on_input(Message::SearchString)
                            .on_submit(Message::Search)
                            .padding(5),
                        checkbox("case sensitive", self.case_sensitive)
                            .on_toggle(Message::ToggleCaseSensitive),
                        button(text("Search")).on_press(Message::Search),
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
    LoadError(String),
    SaveError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadError(s) => write!(f, "LoadError: {s}"),
            Self::SaveError(s) => write!(f, "LoadError: {s}"),
        }
    }
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
