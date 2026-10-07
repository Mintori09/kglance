use crate::app::Message;
use crate::core::TypstState;
use crate::ui::components::code_viewer::VirtualCodeViewer;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::theme::font::get_code_font;
use iced::Element;

pub fn view_typst<'a>(
    state: &'a TypstState,
    pdf_state: &'a crate::core::PdfState,
    theme: crate::ui::theme::AppTheme,
    font_size: f32,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    if state.show_source || state.error.is_some() {
        let font = get_code_font(font_family_mono);
        let viewer =
            VirtualCodeViewer::<Message>::new(&state.source_text.document, font_size, font, theme)
                .display_map(&state.source_text.display_map)
                .wrap(word_wrap)
                .tokens(
                    &state.source_text.cached_tokens,
                    state.source_text.cached_tokens_start_line,
                )
                .selection(state.source_text.selection)
                .on_select(|sel| crate::app::messages::TypstMsg::SelectionChanged(sel).into())
                .on_copy(|text| crate::app::messages::ActionMsg::CopyText(text).into());

        let editor_pane = scroll_pane("typst_source_scroll", viewer)
            .filter_wheel(true)
            .container_padding(0.0)
            .on_scroll(|vp| crate::app::messages::TypstMsg::SourceScrolled(vp).into())
            .on_wheel(|delta| crate::app::messages::TypstMsg::SourceWheelScrolled(delta).into())
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
        crate::ui::views::view_pdf(pdf_state, font_size, theme)
    }
}
