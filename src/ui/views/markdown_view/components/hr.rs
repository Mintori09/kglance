use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::color::base::BaseColors;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_horizontal_rule(is_dark: bool) -> Element<'static, Message> {
    let p = BaseColors::palette_for(is_dark);
    container(
        container(text("").size(1))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(p.border.into()),
                ..Default::default()
            })
            .height(STYLE.general.divider_height)
            .width(Length::Fill),
    )
    .padding(STYLE.hr.padding)
    .width(Length::Fill)
    .into()
}
