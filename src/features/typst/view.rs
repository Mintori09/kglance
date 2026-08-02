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
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    if state.show_source || state.error.is_some() {
        let font = get_code_font(font_family_mono);
        let editor = code_editor(
            &state.source_content,
            "typ",
            is_dark,
            font_size,
            font,
            ignore_editor_action,
        );

        let editor_pane = scroll_pane("typst_source_scroll", editor)
            .container_padding(4.0)
            .build();

        if let Some(err_msg) = &state.error {
            let banner = iced::widget::container(
                iced::widget::column![
                    iced::widget::text("Typst Compilation Warning / Error:")
                        .size(13.0)
                        .style(move |_: &iced::Theme| iced::widget::text::Style {
                            color: Some(if is_dark {
                                iced::Color::from_rgb(0.95, 0.4, 0.4)
                            } else {
                                iced::Color::from_rgb(0.8, 0.2, 0.2)
                            }),
                        }),
                    iced::widget::text(err_msg)
                        .size(11.0)
                        .style(move |_: &iced::Theme| iced::widget::text::Style {
                            color: Some(if is_dark {
                                iced::Color::from_rgb(0.8, 0.8, 0.8)
                            } else {
                                iced::Color::from_rgb(0.3, 0.3, 0.3)
                            }),
                        })
                ]
                .spacing(4),
            )
            .padding(8.0)
            .width(iced::Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(if is_dark {
                    iced::Background::Color(iced::Color::from_rgba(0.3, 0.1, 0.1, 0.5))
                } else {
                    iced::Background::Color(iced::Color::from_rgba(1.0, 0.9, 0.9, 1.0))
                }),
                border: iced::Border {
                    color: if is_dark {
                        iced::Color::from_rgb(0.6, 0.2, 0.2)
                    } else {
                        iced::Color::from_rgb(0.9, 0.6, 0.6)
                    },
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
        crate::ui::views::pdf_view::view_pdf(&state.pdf, font_size, is_dark)
    }
}
