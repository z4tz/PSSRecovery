mod systempoller;

use std::collections::HashMap;
use iced::{Center, Element, Subscription, Task};
use iced::futures::channel::mpsc::Sender;
use iced::Theme;
use iced::widget::{text, column, button, Row, row};
use crate::systempoller::{SystemInfo, testpoller, Event};


#[derive(Default, Clone)]
struct Child {
    system_info: SystemInfo
}
impl Child {
    fn new(name: String) -> Child {
        Child {
            system_info: SystemInfo::new(name)
        }
    }

    fn view(&self) -> Element<Message> {
        let labels = column![text("PLC ETHs:"), text("PLC nodes:"), text("Active alarms:")];

        let eth_string = format!("{}/{}", &self.system_info.eth_responding_count(), &self.system_info.eth_count() );
        let nodes_string = format!("{}/{}", &self.system_info.nodes_responding_count(), &self.system_info.nodes_count());
        let values = column![text(eth_string), text(nodes_string), text(self.system_info.active_alarms())];
        let content = row!(labels, values);
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


struct Parent {
    children: HashMap<String, Child>,
    state: State
}

impl Parent {

    fn new() -> (Self, Task<Message>) {
        (Parent {
            children: HashMap::new(),
            state: State::Loading
        }, Task::none())
    }

    fn view(&self) -> Row<Message> {
        match self.state {
            State::Loading => row!["Loading..."],
            State::Running(_) => {row(self.children.values().map(|child /* Type */| child.view())).into()}
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
                        let child = self.children
                            .entry(system_info.name.clone())
                            .or_insert(Child::new(system_info.name.clone()));
                        child.system_info = system_info;

                    }

                }
            }
            Message::Reset(id) => {

                match &mut self.state {
                    State::Running(sender) => {
                        let _ = sender.try_send(id).unwrap();
                    }
                    State::Loading => {println!("No sender found")}
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(testpoller).map(Message::Data)
    }
}


fn main() -> iced::Result {
    iced::application("Testing composed modules", Parent::update, Parent::view)
        .theme(|_| Theme::Light).centered()
        .subscription(Parent::subscription)
        .run_with(Parent::new)
}
