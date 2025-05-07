use iced::{mouse};
use iced::widget::{canvas, Canvas};
use iced::{Color, Rectangle, Renderer, Theme};

#[derive(Debug)]
enum Status {
    Fault,
    Warning,
    Normal
}

#[derive(Debug)]
pub struct StatusLed {
    radius: f32,
    status: Status
}

impl StatusLed {
    fn new<Message>(size: f32, status: Status) -> Canvas<StatusLed, Message> {
        canvas(StatusLed {radius: size/2.0, status}).width(size).height(size)
    }
    pub fn fault<Message>(size: f32) -> Canvas<StatusLed, Message> {
        Self::new(size, Status::Fault)
    }
    pub fn warning<Message>(size: f32) -> Canvas<StatusLed, Message> {
        Self::new(size, Status::Warning)
    }
    pub fn normal<Message>(size: f32) -> Canvas<StatusLed, Message> {
        Self::new(size, Status::Normal)
    }
}

impl<Message> canvas::Program<Message> for StatusLed {

    type State = ();
    
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor
    ) -> Vec<canvas::Geometry> {

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        
        let border = canvas::Path::circle(frame.center(),self.radius-1.0);
        let circle = canvas::Path::circle(frame.center(),self.radius-3.0);

        let background_color = match self.status {
            Status::Fault => {Color::from_rgb(1.0, 0.0, 0.0)}
            Status::Warning => {Color::from_rgb(1.0, 0.6471, 0.0)}
            Status::Normal => {Color::from_rgb(0.0, 1.0, 0.0)}
        };

        frame.fill(&border, Color::BLACK);
        frame.fill(&circle, background_color);

        vec![frame.into_geometry()]
    }
}