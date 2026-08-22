use iced::{
    Alignment, Element, Length, Padding,
    widget::{Space, Stack, button, column, container, row, text},
};

use crate::app::Message;
use crate::core::KglanceState;
use crate::ui::theme::color::primitive::OVERLAY_SHADOW;
use crate::ui::theme::{default_raised, default_root};

use std::borrow::Cow;
use std::path::Path;

fn left_metadata_text(state: &KglanceState) -> String {
    let mut parts: Vec<Cow<'_, str>> = Vec::new();

    if let Some(folder_name) = Path::new(&state.file_name)
        .parent()
        .and_then(|p| p.file_name())
    {
        let name = folder_name.to_string_lossy();
        if !name.is_empty() {
            parts.push(Cow::Owned(format!("{}", name)));
        }
    }

    if !state.file_type_text.is_empty() {
        parts.push(Cow::Borrowed(&state.file_type_text));
    }

    if !state.file_modified_text.is_empty() {
        parts.push(Cow::Borrowed(&state.file_modified_text));
    }

    let is_markdown = file_has_extension(state, "md") || file_has_extension(state, "markdown");
    let stats = if is_markdown {
        Some((
            state.markdown.word_count,
            state.markdown.char_count,
            state.markdown.reading_time_mins,
        ))
    } else if state.text.word_count > 0 {
        Some((
            state.text.word_count,
            state.text.char_count,
            state.text.reading_time_mins,
        ))
    } else {
        None
    };

    if let Some((words, chars, reading_mins)) = stats
        && words > 0
    {
        parts.push(Cow::Owned(format!("{} words ({} chars)", words, chars)));
        let mins = if reading_mins == 0 { 1 } else { reading_mins };
        parts.push(Cow::Owned(format!("{} min read", mins)));
    }

    parts.join(" • ")
}

fn metadata_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.weak.text),
    }
}

fn footer<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    let left = left_metadata_text(state);
    let right = &state.file_size_text;

    if left.is_empty() && right.is_empty() {
        return container(text("")).padding(0).into();
    }

    let counter = playlist_position_button(state);
    let page_counter = page_indicator(state);
    let typst = typst_toggle_button(state);

    let left_row = row![
        counter,
        page_counter,
        text(left).size(11).style(metadata_style),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let right_row = row![
        text(right).size(11).style(metadata_style),
        typst,
        setting_button(),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(
        row![left_row, Space::new().width(Length::Fill), right_row]
            .align_y(Alignment::Center)
            .padding([4, 12]),
    )
    .width(Length::Fill)
    .style(default_raised)
    .into()
}

fn playlist_position_button<'a>(state: &KglanceState) -> Option<Element<'a, Message>> {
    (state.playlist.len() > 1).then(|| {
        button(
            text(format!(
                "[ {} / {} ]",
                state.current_index + 1,
                state.playlist.len()
            ))
            .size(11),
        )
        .on_press(crate::app::messages::NavigationMsg::ToggleViewMode.into())
        .style(iced::widget::button::secondary)
        .padding([2, 6])
        .into()
    })
}

fn file_has_extension(state: &KglanceState, extension: &str) -> bool {
    std::path::Path::new(&state.file_name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

fn page_indicator<'a>(state: &KglanceState) -> Option<Element<'a, Message>> {
    let (page, total) = if file_has_extension(state, "pdf") || state.file_type_text.contains("PDF")
    {
        if state.pdf.page_count == 0 {
            return None;
        }

        (
            state
                .pdf
                .visible_page
                .load(std::sync::atomic::Ordering::Relaxed)
                + 1,
            state.pdf.page_count,
        )
    } else if file_has_extension(state, "typ") || state.file_type_text.contains("Typst") {
        if state.typst.pdf.page_count == 0 {
            return None;
        }

        (
            state
                .typst
                .pdf
                .visible_page
                .load(std::sync::atomic::Ordering::Relaxed)
                + 1,
            state.typst.pdf.page_count,
        )
    } else {
        return None;
    };

    Some(
        text(format!("[ {page} / {total} ]"))
            .size(11)
            .style(metadata_style)
            .into(),
    )
}

fn setting_button<'a>() -> Element<'a, Message> {
    iced::widget::button(text("⚙").size(12).style(metadata_style))
        .on_press(crate::app::messages::NavigationMsg::ToggleSettingsClicked.into())
        .style(iced::widget::button::secondary)
        .padding([2, 6])
        .into()
}

fn typst_toggle_button<'a>(state: &KglanceState) -> Option<Element<'a, Message>> {
    if state.file_name.to_lowercase().ends_with(".typ") {
        let label = if state.typst.show_source {
            "👁 Rendered"
        } else {
            "</> Source"
        };
        Some(
            iced::widget::button(text(label).size(11).style(metadata_style))
                .on_press(crate::app::messages::TypstMsg::ToggleSource.into())
                .style(iced::widget::button::secondary)
                .padding([2, 8])
                .into(),
        )
    } else {
        None
    }
}

fn content<'a>(preview_body: Element<'a, Message>, edge_to_edge: bool) -> Element<'a, Message> {
    let padding = if edge_to_edge { 0 } else { 10 };
    container(preview_body)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(padding)
        .into()
}

fn toasts<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    if state.toasts.is_empty() {
        return Element::from(container(text("")).padding(0));
    }

    let items: Vec<Element<'a, Message>> = state
        .toasts
        .iter()
        .map(|t| {
            container(text(&t.message).size(13))
                .padding(Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0,
                })
                .style(|theme: &iced::Theme| {
                    use iced::widget::container;
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.base.color.into()),
                        text_color: Some(palette.background.base.text),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 0.0,
                            color: iced::Color::TRANSPARENT,
                        },
                        shadow: iced::Shadow {
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 8.0,
                            color: OVERLAY_SHADOW,
                        },
                        ..Default::default()
                    }
                })
                .into()
        })
        .collect();

    column(items)
        .spacing(6)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 24.0,
            left: 0.0,
        })
        .width(Length::Shrink)
        .into()
}

pub fn view_window<'a>(
    state: &'a KglanceState,
    preview_body: Element<'a, Message>,
    edge_to_edge: bool,
) -> Element<'a, Message> {
    let show_settings_modal = matches!(state.view_mode, crate::core::ViewMode::Settings);

    let main_body: Element<'a, Message> = match &state.view_mode {
        crate::core::ViewMode::Grid(thumbnails) => crate::ui::views::view_grid(
            thumbnails,
            state.current_index,
            state.grid_scale,
            state.grid_search_visible,
            &state.grid_search_query,
        ),
        _ => content(preview_body, edge_to_edge),
    };

    let layout = column![
        container(main_body)
            .width(Length::Fill)
            .height(Length::Fill),
        footer(state)
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let base = container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(default_root);

    let toast_layer = container(toasts(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End)
        .align_x(iced::alignment::Horizontal::Center);

    let mut stack = Stack::new().push(base).push(toast_layer);

    if show_settings_modal {
        let dummy_config: &'static crate::core::config::UiConfig =
            Box::leak(Box::new(crate::core::config::UiConfig {
                theme: Some(state.theme_setting.clone()),
                font_size: state.font_size,
                font_family: state.font_family.clone(),
                font_family_mono: state.font_family_mono.clone(),
                epub_font_family: state.epub_font_family.clone(),
                max_text_width: state.max_text_width,
                default_width: state.window_default_size.width as u32,
                default_height: state.window_default_size.height as u32,
                min_width: state.window_min_size.width as u32,
                min_height: state.window_min_size.height as u32,
                prefer_mermaid_cli: state.prefer_mermaid_cli,
                word_wrap: state.word_wrap,
                json_tree_view: state.json_tree_view,
            }));

        let static_fonts: &'static [String] =
            Box::leak(crate::ui::views::setting_page::get_system_fonts().into_boxed_slice());
        let theme = if state.app_theme.is_dark() {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        };

        let settings_content =
            crate::ui::views::setting_page::settings_page(&theme, dummy_config, static_fonts);

        let modal_box = container(iced::widget::scrollable(settings_content))
            .max_width(550.0)
            .max_height(500.0);

        let backdrop = iced::widget::opaque(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(Alignment::Center)
                .style(|_| container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                    ..Default::default()
                }),
        );

        stack = stack.push(backdrop);
    }

    stack.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_text_full() {
        let state = KglanceState {
            file_name: "../../testing-file/markdown.md".to_string(),
            file_type_text: "Markdown Document".to_string(),
            file_size_text: "12.4 KB".to_string(),
            file_modified_text: "2026-07-22".to_string(),
            ..Default::default()
        };

        let left = left_metadata_text(&state);
        assert_eq!(left, "testing-file • Markdown Document • 2026-07-22");
        assert_eq!(state.file_size_text, "12.4 KB");
    }

    #[test]
    fn test_metadata_text_partial() {
        let state = KglanceState {
            file_name: "test_doc.md".to_string(),
            file_type_text: "Text Document".to_string(),
            ..Default::default()
        };

        let left = left_metadata_text(&state);
        assert_eq!(left, "Text Document");
    }
}
