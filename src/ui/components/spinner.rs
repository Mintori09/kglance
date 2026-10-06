use std::f32::consts::PI;

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Radians, Rectangle, Theme};

pub struct Spinner {
    pub angle_deg: f32,
    pub radius: f32,
    pub stroke_width: f32,
    pub color: Color,
    pub track_color: Color,
}

impl Spinner {
    pub fn new(angle_deg: f32, color: Color, track_color: Color) -> Self {
        Self {
            angle_deg,
            radius: 20.0,
            stroke_width: 3.5,
            color,
            track_color,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn stroke_width(mut self, stroke_width: f32) -> Self {
        self.stroke_width = stroke_width;
        self
    }
}

impl<Message> canvas::Program<Message> for Spinner {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();

        // Draw track circle (subtle background ring)
        let track_circle = Path::circle(center, self.radius);
        frame.stroke(
            &track_circle,
            Stroke::default()
                .with_color(self.track_color)
                .with_width(self.stroke_width),
        );

        // Draw spinning arc (~120 degrees arc)
        let start_angle = self.angle_deg.to_radians();
        let end_angle = start_angle + (PI * 2.0 / 3.0);

        let arc = Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius: self.radius,
                start_angle: Radians(start_angle),
                end_angle: Radians(end_angle),
            });
        });

        frame.stroke(
            &arc,
            Stroke::default()
                .with_color(self.color)
                .with_width(self.stroke_width)
                .with_line_cap(canvas::LineCap::Round),
        );

        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new(
            45.0,
            Color::from_rgb(0.2, 0.5, 0.9),
            Color::from_rgba(0.2, 0.5, 0.9, 0.15),
        )
        .radius(24.0)
        .stroke_width(4.0);

        assert_eq!(spinner.angle_deg, 45.0);
        assert_eq!(spinner.radius, 24.0);
        assert_eq!(spinner.stroke_width, 4.0);
    }
}
