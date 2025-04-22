use iced::{mouse};
use iced::widget::{canvas, Canvas};
use iced::{Color, Rectangle, Renderer, Theme};

#[derive(Debug)]
pub enum Status {
    Fault,
    Warning,
    Normal
}

#[derive(Debug)]
pub struct StatusBox {
    radius: f32,
    status: Status
}

impl StatusBox {
    pub fn new(radius: f32, status: Status) -> StatusBox {
        StatusBox {radius, status}
    }
}

impl<Message> canvas::Program<Message> for StatusBox {
    
    // No internal state
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
        
        // We create a `Path` representing a simple circle
        let border = canvas::Path::circle(frame.center(),self.radius);
        let circle = canvas::Path::circle(frame.center(),self.radius-2.0);

        let background_color = match self.status {
            Status::Fault => {Color::from_rgb(1.0, 0.0, 0.0)}
            Status::Warning => {Color::from_rgb(1.0, 0.6471, 0.0)}
            Status::Normal => {Color::from_rgb(0.0, 1.0, 0.0)}
        };
        
        // And fill it with some color
        frame.fill(&border, Color::BLACK);
        frame.fill(&circle, background_color);

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }
}

pub fn status_box<Message>(size: f32, status: Status) -> Canvas<StatusBox, Message> {
    canvas(StatusBox::new(size/2.0, status)).width(size).height(size)
}