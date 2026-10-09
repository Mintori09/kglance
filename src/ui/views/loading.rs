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
    let palette = app_theme.palette();
    let base_color = palette.base.text;
    let accent_color = palette.roles.accent;
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
        Space::new().height(Length::Fixed(crate::ui::theme::tokens::spacing::L)),
        text(label)
            .size(crate::ui::theme::tokens::typography::BODY_LG)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_color),
            }),
    ]
    .align_x(Alignment::Center)
    .spacing(crate::ui::theme::tokens::spacing::XS);

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
