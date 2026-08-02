use crate::app::Message;
use crate::core::TypstState;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::theme::font::get_code_font;
use iced::Element;
use iced::widget::text_editor::Action;

fn ignore_editor_action(_: Action) -> Message {
    Message::None
}

pub fn view_typst<'a>(
    state: &'a TypstState,
    theme: crate::ui::theme::AppTheme,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    if state.show_source || state.error.is_some() {
        let font = get_code_font(font_family_mono);
        let editor = code_editor(
            &state.source_content,
            "typ",
            theme,
            font_size,
            font,
            ignore_editor_action,
        );

        let editor_pane = scroll_pane("typst_source_scroll", editor)
            .container_padding(4.0)
            .build();

        if let Some(err_msg) = &state.error {
            let roles = theme.palette().roles;
            let base = theme.palette().base;
            let banner = iced::widget::container(
                iced::widget::column![
                    iced::widget::text("Typst Compilation Warning / Error:")
                        .size(13.0)
                        .style(move |_: &iced::Theme| iced::widget::text::Style {
                            color: Some(roles.danger),
                        }),
                    iced::widget::text(err_msg)
                        .size(11.0)
                        .style(move |_: &iced::Theme| iced::widget::text::Style {
                            color: Some(base.text_dim),
                        })
                ]
                .spacing(4),
            )
            .padding(8.0)
            .width(iced::Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(base.surface_raised)),
                border: iced::Border {
                    color: roles.danger,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

            iced::widget::column![banner, editor_pane].spacing(6).into()
        } else {
            editor_pane
        }
    } else {
        crate::ui::views::view_pdf(&state.pdf, font_size, theme)
    }
}
