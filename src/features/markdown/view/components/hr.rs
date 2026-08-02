use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::AppTheme;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_horizontal_rule(theme: AppTheme) -> Element<'static, Message> {
    let p = theme.palette().base;
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
