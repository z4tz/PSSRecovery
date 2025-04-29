mod systempoller;
mod statusbox;

use std::collections::HashMap;
use iced::{Center, Color, Element, Length, Subscription, Task};
use iced::futures::channel::mpsc::Sender;
use iced::Theme;
use iced::widget::{text, column, button, row, container, stack, opaque, mouse_area, center, scrollable, Row, Column, horizontal_space, vertical_space};
use iced::clipboard;
use rfd::{AsyncFileDialog};
use crate::systempoller::{SystemInfo, testpoller, Event, BackgroundMessage};
use crate::statusbox::{ status_box, Status};

#[derive(Debug, Clone)]
enum Message {
    Data(Event),
    Reset(String),
    ResetAll,
    ShowPopup(PopupState),
    HidePopup,
    CopyPopupText,
    FileDialog,
    LoadConfig(Option<String>),
}
enum State {
    Loading,
    Running(Sender<BackgroundMessage>),
}
#[derive(Debug, Clone)]
enum PopupState {
    Hidden,
    ShowSystem(String),
    ShowAll,
    ShowError(String),
}

struct RecoveryApp {
    system_map: HashMap<String, SystemInfo>,
    state: State,
    popup_state: PopupState
}

impl RecoveryApp {
    fn new() -> (Self, Task<Message>) {
        (RecoveryApp {
            system_map: HashMap::new(),
            state: State::Loading,
            popup_state: PopupState::Hidden,
        }, Task::none())
    }

    fn view(&self) -> Element<Message> {
        match self.state {
            State::Loading => row!["Waiting on background thread"].into(),
            State::Running(_) => {
                
                //top row with buttons
                let mut column = Column::new().width(Length::Fill).align_x(Center);
                let load_button = button("Load config").on_press(Message::FileDialog);
                let reset_button = match self.system_map.is_empty() {
                    false => button("Reset all").on_press(Message::ResetAll),
                    true => button("Reset all")
                };
                let host_info_button = match self.system_map.is_empty() {
                    false => button("All hosts info").on_press(Message::ResetAll),
                    true => button("All hosts info")
                };

                let button_row = row![
                    load_button,
                    reset_button,
                    host_info_button
                ].spacing(10);
                column = column.push(button_row);
                
                //system views
                if self.system_map.is_empty() {
                    column = column.push(text("Waiting for config to be loaded..."))
                }
                else {
                    let mut row = Row::new();
                    for (i, system_info) in self.sorted_systems().iter().enumerate() {
                        row = row.push(system_view(system_info));
                        if i%4 == 3 {
                            column = column.push(row);
                            row = Row::new();
                        }
                    }
                    column = column.push(row);
                }

                let content = container(column).width(Length::Fill).height(Length::Fill);
                
                match &self.popup_state {
                    PopupState::Hidden => {
                        content.into()
                    }
                    PopupState::ShowError(error_message) => {
                        let popup = container(
                            column!(
                                text("Error loading file:" ).size(20),
                                text(error_message),
                                row!(
                                    horizontal_space(),
                                    button("OK").on_press(Message::HidePopup),
                                )
                            )
                        ).width(500).height(400).style(container::rounded_box).padding(10);
                        modal(content, popup, Message::HidePopup)
                    }
                    _ => {  // showSystem and showAll
                        let popup = container(
                            column!(
                            text("Hosts not responding:" ).size(20),
                            scrollable(text(self.popup_text()).width(Length::Fill).size(15)).height(Length::Fill),
                            row!(
                                button("Copy text").on_press(Message::CopyPopupText),
                                horizontal_space(),
                                button("OK").on_press(Message::HidePopup),
                                )
                            ).spacing(10)
                        ).width(500).height(400).style(container::rounded_box).padding(10);
                        modal(content, popup, Message::HidePopup)
                    }
                }
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Data(event) => {
                match event {
                    Event::Setup(sender) => {
                        self.state = State::Running(sender);
                        Task::none()
                    }
                    Event::Update(system_info) => {
                        self.system_map.insert(system_info.name.clone(), system_info);
                        Task::none()
                    }
                    Event::FileError(error_message) => {
                        self.popup_state = PopupState::ShowError(error_message);
                        Task::none()
                    }
                }
            }
            
            Message::Reset(system_name) => {
                match &mut self.state {
                    State::Running(sender) => {
                        let _ = sender.try_send(BackgroundMessage::Reset(system_name)).unwrap();
                        Task::none()
                    }
                    State::Loading => {Task::none()}
                }
            }
            
            Message::ResetAll => {
                match &mut self.state {
                    State::Running(sender) => {
                        let _ = sender.try_send(BackgroundMessage::ResetAll).unwrap();
                        Task::none()    
                    }
                    State::Loading => {Task::none()}
                }
            }
            
            Message::ShowPopup(popup_state) => {
                self.popup_state = popup_state;
                Task::none()
            }
            
            Message::HidePopup => {
                self.popup_state = PopupState::Hidden;
                Task::none()
            }
            
            Message::CopyPopupText => {
                clipboard::write(self.popup_text())
            }
            Message::FileDialog => {
                Task::perform(get_filename(),Message::LoadConfig)
            }
            Message::LoadConfig(fileoption) => {
                self.system_map.clear();
                match &mut self.state {
                    State::Loading => {}
                    State::Running(sender) => {
                        match fileoption {
                            None => {}
                            Some(filename) => {
                                let _ = sender.try_send(BackgroundMessage::LoacFile(filename));
                            }
                        }
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(testpoller).map(Message::Data)
    }

    fn popup_text(&self) -> String {
        match &self.popup_state {
            PopupState::Hidden => {"".to_string()}
            PopupState::ShowSystem(system_name) => {
                self.system_map[system_name].failed_hosts()
            }
            PopupState::ShowAll => {
                self.sorted_systems().iter()
                    .filter(|system| !system.eths_ok() || !system.nodes_ok())
                    .map(|system| system.failed_hosts())
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            _ => {"".to_string()}
        }
    }
    fn sorted_systems(&self) -> Vec<&SystemInfo> {
        let mut system_info_vec: Vec<_> = self.system_map.values().collect();
        system_info_vec.sort_by(|a, b|a.name.cmp(&b.name));
        system_info_vec
    }
}

// view for a system_info
fn system_view(system_info: &SystemInfo) -> Element<Message> {
    let labels = column![text("PLC ETHs:"), text("PLC nodes:"), text("Active alarms:")];

    let active_alarms_text = match system_info.active_alarms() {
        None => {"Unknown".to_string()}
        Some(active_alarms) => {active_alarms.to_string()}
    };

    let values = column![text(system_info.eth_status()), text(system_info.nodes_status()), text(active_alarms_text)];

    let eth_status = match system_info.eths_ok() {
        true => {Status::Normal}
        false => {Status::Fault}
    };
    let nodes_status = match system_info.nodes_ok() {
        true => {Status::Normal}
        false => {Status::Fault}
    };
    let active_alarms_status = match system_info.active_alarms() {
        None => {Status::Fault}
        Some(value) => {
            match value {
                true => {Status::Warning}
                false => {Status::Normal}
            }
        }
    };

    let statusbox_size = 20.0;
    let eth_statusbox = status_box(statusbox_size, eth_status);
    let nodes_statusbox = status_box(statusbox_size, nodes_status);
    let alarms_statusbox = status_box(statusbox_size, active_alarms_status);
    let status_boxes = column![eth_statusbox, nodes_statusbox, alarms_statusbox];

    let content = row!(labels, values, status_boxes).spacing(5);

    let reset_button = button("Reset alarms").on_press(Message::Reset(system_info.name.clone()));
    let hosts_info_button = button("Hosts info").on_press(Message::ShowPopup(PopupState::ShowSystem(system_info.name.to_string())));
    let button_row = row![reset_button, hosts_info_button].spacing(10);

    column![text(&system_info.name).size(20), content,vertical_space().height(Length::Fixed(5.0)), button_row].align_x(Center).padding(20).into()
}

// used for popup
fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message> where Message: Clone + 'a, {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(Color {a: 0.8,..Color::BLACK}.into(),),..container::Style::default()}
            })).on_press(on_blur)
        )
    ].into()
}

async fn get_filename() -> Option<String> {
    let file = AsyncFileDialog::new()
        .set_title("Open config file...")
        .pick_file()
        .await;

    match file {
        None => {None}
        Some(handle) => {Some(handle.inner().to_str()?.to_string())}
    }
}

fn main() -> iced::Result {
    iced::application("PSS PLC recovery program", RecoveryApp::update, RecoveryApp::view)
        .theme(|_| Theme::Light).centered()
        .subscription(RecoveryApp::subscription)
        .antialiasing(true)
        .run_with(RecoveryApp::new)
}
