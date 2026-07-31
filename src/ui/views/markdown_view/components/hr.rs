use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::color::base::BaseColors;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_horizontal_rule() -> Element<'static, Message> {
    container(
        container(text("").size(1))
            .style(|theme: &iced::Theme| {
                let p = BaseColors::palette(theme);
                container::Style {
                    background: Some(p.border.into()),
                    ..Default::default()
                }
            })
            .height(STYLE.general.divider_height)
            .width(Length::Fill),
    )
    .padding(STYLE.hr.padding)
    .width(Length::Fill)
    .into()
}
