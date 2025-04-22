mod systempoller;
mod statusbox;

use std::collections::HashMap;
use iced::{Center, Element, Subscription, Task};
use iced::futures::channel::mpsc::Sender;
use iced::Theme;
use iced::widget::{text, column, button, Row, row};
use crate::systempoller::{SystemInfo, testpoller, Event};
use crate::statusbox::{ status_box, Status};



#[derive(Default, Clone)]
struct System_Widget {
    system_info: SystemInfo
}
impl System_Widget {
    fn new(name: String) -> System_Widget {
        System_Widget {
            system_info: SystemInfo::new(name)
        }
    }

    fn view(&self) -> Element<Message> {
        let labels = column![text("PLC ETHs:"), text("PLC nodes:"), text("Active alarms:")];

        let active_alarms_text = match self.system_info.active_alarms() {
            None => {"Unknown".to_string()}
            Some(active_alarms) => {active_alarms.to_string()}
        };
        
        
        let values = column![text(self.system_info.eth_status()), text(self.system_info.nodes_status()), text(active_alarms_text)];
        
        let eth_status = match self.system_info.eths_ok() {
            true => {Status::Normal}
            false => {Status::Fault}
        };
        let nodes_status = match self.system_info.nodes_ok() {
            true => {Status::Normal}
            false => {Status::Fault}
        };
        let active_alarms_status = match self.system_info.active_alarms() {
            None => {Status::Fault}
            Some(value) => {match value {
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
        
        let content = row!(labels, values, status_boxes);
        let reset_button = button("Reset").on_press(Message::Reset(self.system_info.name.clone()));

        column![text(&self.system_info.name).size(20), content, reset_button].align_x(Center).into()
    }
}


#[derive(Debug, Clone)]
enum Message {
    Data(Event),
    Reset(String)

}
enum State {
    Loading,
    Running(Sender<String>),
}


struct SystemsRecovery {
    systemes: HashMap<String, System_Widget>,
    state: State
}

impl SystemsRecovery {

    fn new() -> (Self, Task<Message>) {
        (SystemsRecovery {
            systemes: HashMap::new(),
            state: State::Loading
        }, Task::none())
    }

    fn view(&self) -> Row<Message> {
        match self.state {
            State::Loading => row!["Loading..."],
            State::Running(_) => {row(self.systemes.values().map(|child /* Type */| child.view())).into()}
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Data(event) => {
                match event {
                    Event::Setup(sender) => {
                        self.state = State::Running(sender);
                    }
                    Event::Update(system_info) => {
                        let child = self.systemes
                            .entry(system_info.name.clone())
                            .or_insert(System_Widget::new(system_info.name.clone()));
                        child.system_info = system_info;

                    }

                }
            }
            Message::Reset(id) => {
                match &mut self.state {
                    State::Running(sender) => {
                        let _ = sender.try_send(id).unwrap();
                    }
                    State::Loading => {}
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(testpoller).map(Message::Data)
    }
}

fn main() -> iced::Result {
    iced::application("PSS PLC recovery program", SystemsRecovery::update, SystemsRecovery::view)
        .theme(|_| Theme::Light).centered()
        .subscription(SystemsRecovery::subscription)
        .run_with(SystemsRecovery::new)
}
