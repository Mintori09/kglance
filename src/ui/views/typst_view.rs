use crate::app::Message;
use crate::core::TypstState;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::theme::font::get_code_font;
use crate::ui::views::pdf_view::view_pdf_pages;
use iced::widget::text_editor::Action;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::ui::theme::tokens::spacing;

const TOOLBAR_PADDING: f32 = spacing::XS;
const TOOLBAR_SPACING: f32 = spacing::S;
const SOURCE_SCROLL_PANE_ID: &str = "typst_source_scroll";
const PAGES_SCROLL_PANE_ID: &str = "typst_pages_scroll";
const TOOLBAR_TEXT_SIZE: f32 = 13.0;

fn ignore_editor_action(_: Action) -> Message {
    Message::None
}

pub fn view_typst<'a>(
    state: &'a TypstState,
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    let toolbar = build_toolbar(state.show_source);

    let body: Element<'a, Message> = if state.show_source {
        let font = get_code_font(font_family_mono);
        let editor = code_editor(
            &state.source_content,
            "typ",
            is_dark,
            font_size,
            font,
            ignore_editor_action,
        );
        scroll_pane(SOURCE_SCROLL_PANE_ID, editor)
            .container_padding(TOOLBAR_PADDING)
            .build()
    } else {
        view_pdf_pages(&state.pdf, PAGES_SCROLL_PANE_ID, |vp| {
            crate::app::messages::TypstMsg::Scrolled(vp).into()
        })
    };

    column![toolbar, body].height(Length::Fill).into()
}

fn build_toolbar(show_source: bool) -> Element<'static, Message> {
    let rendered_button = button(text("Rendered").size(TOOLBAR_TEXT_SIZE));
    let source_button = button(text("Source").size(TOOLBAR_TEXT_SIZE));

    let rendered_button = if show_source {
        rendered_button.on_press(crate::app::messages::TypstMsg::ToggleSource.into())
    } else {
        rendered_button
    };
    let source_button = if show_source {
        source_button
    } else {
        source_button.on_press(crate::app::messages::TypstMsg::ToggleSource.into())
    };

    let toolbar = row![]
        .spacing(TOOLBAR_SPACING)
        .padding(TOOLBAR_PADDING)
        .push(rendered_button)
        .push(source_button)
        .push(
            text(if show_source { "Source" } else { "Rendered" })
                .size(TOOLBAR_TEXT_SIZE)
                .width(Length::Fill)
                .center(),
        );

    container(toolbar)
        .width(Length::Fill)
        .padding(TOOLBAR_PADDING)
        .into()
}
