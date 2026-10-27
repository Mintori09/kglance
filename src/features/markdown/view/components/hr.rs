use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::AppTheme;
use iced::alignment;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_horizontal_rule(
    theme: AppTheme,
    is_scene_break: bool,
) -> Element<'static, Message> {
    let p = theme.palette().base;

    if is_scene_break {
        let mut dim = p.text_dim;
        dim.a *= 0.65;
        container(
            text("•   •   •")
                .size(13.0)
                .color(dim)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .padding([14, 0])
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
    } else {
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
}
