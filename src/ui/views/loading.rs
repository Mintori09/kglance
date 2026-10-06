use std::path::Path;

use iced::widget::canvas::Canvas;
use iced::widget::{Space, column, container, text};
use iced::{Alignment, Color, Element, Length};

use crate::app::Message;
use crate::ui::components::spinner::Spinner;
use crate::ui::theme::AppTheme;

pub fn view_loading<'a>(
    file_name: &str,
    angle_deg: f32,
    app_theme: AppTheme,
) -> Element<'a, Message> {
    let base_color = match app_theme {
        AppTheme::Dark => Color::from_rgb(0.9, 0.9, 0.9),
        AppTheme::Light => Color::from_rgb(0.1, 0.1, 0.1),
        AppTheme::Nord => Color::from_rgb(0.88, 0.91, 0.94),
    };
    let accent_color = match app_theme {
        AppTheme::Dark => Color::from_rgb(0.24, 0.51, 0.96),
        AppTheme::Light => Color::from_rgb(0.15, 0.45, 0.85),
        AppTheme::Nord => Color::from_rgb(0.53, 0.75, 0.82),
    };
    let track_color = Color {
        a: 0.15,
        ..accent_color
    };

    let spinner = Canvas::new(
        Spinner::new(angle_deg, accent_color, track_color)
            .radius(20.0)
            .stroke_width(3.5),
    )
    .width(Length::Fixed(56.0))
    .height(Length::Fixed(56.0));

    let display_name = Path::new(file_name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());

    let label = if display_name.is_empty() {
        "Loading preview...".to_string()
    } else {
        format!("Loading \"{display_name}\"...")
    };

    let content = column![
        spinner,
        Space::new().height(Length::Fixed(16.0)),
        text(label)
            .size(14)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_color),
            }),
    ]
    .align_x(Alignment::Center)
    .spacing(4);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(Alignment::Center)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_loading_constructs_without_panic() {
        let _elem = view_loading("sample.docx", 90.0, AppTheme::Dark);
    }
}
