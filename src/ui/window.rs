use iced::{
    Alignment, Element, Length, Padding,
    widget::{Space, Stack, button, column, container, row, text},
};

use crate::app::Message;
use crate::core::KglanceState;
use crate::ui::theme::AppTheme;
use crate::ui::theme::color::primitive::OVERLAY_SHADOW;
use crate::ui::theme::tokens::{elevation, radius, spacing, typography};
use crate::ui::theme::{default_raised, default_root};
use std::path::Path;

fn left_metadata_text(state: &KglanceState) -> String {
    let mut parts = Vec::new();

    if let Some(name) = Path::new(&state.file_name)
        .parent()
        .and_then(|p| p.file_name())
        .filter(|name| !name.is_empty())
    {
        parts.push(name.to_string_lossy().into_owned());
    }

    if !state.file_type_text.is_empty() {
        parts.push(state.file_type_text.clone());
    }

    if !state.file_modified_text.is_empty() {
        parts.push(state.file_modified_text.clone());
    }

    let is_markdown = file_has_extension(state, "md") || file_has_extension(state, "markdown");

    if is_markdown && state.markdown.word_count > 0 {
        let mins = state.markdown.reading_time_mins.max(1);
        parts.push(format!(
            "{} words · {mins} min read",
            state.markdown.word_count
        ));
    } else if state.text.word_count > 0 {
        let mins = state.text.reading_time_mins.max(1);
        parts.push(format!("{} words · {mins} min read", state.text.word_count));
    }

    parts.join(" • ")
}

fn metadata_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let p = crate::ui::theme::color::BaseColors::palette(theme);
    iced::widget::text::Style {
        color: Some(p.text_dim),
    }
}

fn footer<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    let left = left_metadata_text(state);
    let right = &state.file_size_text;

    if left.is_empty() && right.is_empty() {
        return container(text("")).padding(0).into();
    }

    let main_font = crate::ui::theme::font::get_main_font(state.font_family.as_deref());
    let counter = playlist_position_button(state, main_font);
    let page_counter = page_indicator(state, main_font);
    let toc_btn = toc_toggle_button(state, main_font);
    let typst = typst_toggle_button(state, main_font);
    let info = image_info_button(state, main_font);

    let left_row = row![
        counter,
        page_counter,
        toc_btn,
        text(left)
            .size(typography::CAPTION)
            .font(main_font)
            .style(metadata_style),
    ]
    .spacing(spacing::S)
    .align_y(Alignment::Center);

    let right_row = row![
        text(right)
            .size(typography::CAPTION)
            .font(main_font)
            .style(metadata_style),
        typst,
        info,
        setting_button(main_font),
    ]
    .spacing(spacing::S)
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

fn toc_toggle_button<'a>(state: &KglanceState, font: iced::Font) -> Option<Element<'a, Message>> {
    let is_markdown = file_has_extension(state, "md") || file_has_extension(state, "markdown");
    if is_markdown && !state.markdown.toc.is_empty() {
        let style = if state.markdown.toc_visible {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        };
        Some(
            button(text("📑 Outline").size(typography::CAPTION).font(font))
                .on_press(crate::app::messages::MarkdownMsg::TocToggled.into())
                .style(style)
                .padding([2, 6])
                .into(),
        )
    } else if (file_has_extension(state, "epub") || state.file_type_text.contains("EPUB"))
        && !state.epub.markdown_state.toc.is_empty()
    {
        let style = if state.epub.sidebar_visible {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        };
        Some(
            button(text("📑 Chapters").size(typography::CAPTION).font(font))
                .on_press(crate::app::messages::EpubMsg::SidebarToggled.into())
                .style(style)
                .padding([2, 6])
                .into(),
        )
    } else {
        None
    }
}

fn playlist_position_button<'a>(
    state: &KglanceState,
    font: iced::Font,
) -> Option<Element<'a, Message>> {
    (state.playlist.len() > 1).then(|| {
        button(
            text(format!(
                "[ {} / {} ]",
                state.current_index + 1,
                state.playlist.len()
            ))
            .size(11)
            .font(font),
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

fn page_indicator<'a>(state: &KglanceState, font: iced::Font) -> Option<Element<'a, Message>> {
    let is_paged = file_has_extension(state, "pdf")
        || state.file_type_text.contains("PDF")
        || file_has_extension(state, "typ")
        || state.file_type_text.contains("Typst");

    if !is_paged || state.pdf.page_count == 0 {
        return None;
    }

    let page = state
        .pdf
        .visible_page
        .load(std::sync::atomic::Ordering::Relaxed)
        + 1;
    let total = state.pdf.page_count;

    Some(
        text(format!("[ {page} / {total} ]"))
            .size(11)
            .font(font)
            .style(metadata_style)
            .into(),
    )
}

fn setting_button<'a>(font: iced::Font) -> Element<'a, Message> {
    iced::widget::button(
        text("⚙")
            .size(typography::ICON_MD)
            .font(font)
            .style(metadata_style),
    )
    .on_press(crate::app::messages::NavigationMsg::ToggleSettingsClicked.into())
    .style(iced::widget::button::secondary)
    .padding([2, 6])
    .into()
}

fn image_info_button<'a>(state: &KglanceState, font: iced::Font) -> Option<Element<'a, Message>> {
    if !state.image.exif_content.is_empty() {
        let style = if state.image.show_info {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        };
        Some(
            iced::widget::button(text("ℹ").size(typography::ICON_MD).font(font))
                .on_press(crate::app::messages::ImageMsg::ToggleInfo.into())
                .style(style)
                .padding([2, 6])
                .into(),
        )
    } else {
        None
    }
}

fn typst_toggle_button<'a>(state: &KglanceState, font: iced::Font) -> Option<Element<'a, Message>> {
    if state.file_name.to_lowercase().ends_with(".typ") {
        let label = if state.typst.show_source {
            "👁 Rendered"
        } else {
            "</> Source"
        };
        Some(
            iced::widget::button(
                text(label)
                    .size(typography::CAPTION)
                    .font(font)
                    .style(metadata_style),
            )
            .on_press(crate::app::messages::TypstMsg::ToggleSource.into())
            .style(iced::widget::button::secondary)
            .padding([2, 8])
            .into(),
        )
    } else {
        None
    }
}

fn toasts<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    if state.toasts.is_empty() {
        return Element::from(container(text("")).padding(0));
    }

    let main_font = crate::ui::theme::font::get_main_font(state.font_family.as_deref());
    let items: Vec<Element<'a, Message>> = state
        .toasts
        .iter()
        .map(|t| {
            container(text(&t.message).size(typography::BODY_MD).font(main_font))
                .padding(Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0,
                })
                .style(|theme: &iced::Theme| {
                    use iced::widget::container;
                    let p = AppTheme::from(theme).palette().base;
                    container::Style {
                        background: Some(p.surface_raised.into()),
                        text_color: Some(p.text),
                        border: iced::Border {
                            radius: radius::MD.into(),
                            width: 1.0,
                            color: p.border,
                        },
                        shadow: elevation::medium(OVERLAY_SHADOW),
                        ..Default::default()
                    }
                })
                .into()
        })
        .collect();

    column(items)
        .spacing(spacing::XS + 2.0)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: spacing::XL,
            left: 0.0,
        })
        .width(Length::Shrink)
        .into()
}

pub fn view_window<'a>(
    state: &'a KglanceState,
    preview_body: Element<'a, Message>,
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
        _ => preview_body,
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
        let ui_config = crate::core::config::UiConfig {
            theme: Some(state.theme_setting.clone()),
            font_size: state.font_size,
            font_family: state.font_family.clone(),
            font_family_mono: state.font_family_mono.clone(),
            epub_font_family: state.epub_font_family.clone(),
            epub_reading_mode: state.epub_reading_mode,
            max_text_width: state.max_text_width,
            default_width: state.window_default_size.width as u32,
            default_height: state.window_default_size.height as u32,
            min_width: state.window_min_size.width as u32,
            min_height: state.window_min_size.height as u32,
            prefer_mermaid_cli: state.prefer_mermaid_cli,
            word_wrap: state.word_wrap,
            json_tree_view: state.json_tree_view,
        };

        let static_fonts = crate::ui::views::setting_page::get_system_fonts();
        let theme = iced::Theme::from(state.app_theme);

        let settings_content =
            crate::ui::views::setting_page::settings_page(&theme, &ui_config, static_fonts);

        let modal_box = container(iced::widget::scrollable(settings_content))
            .max_width(550.0)
            .max_height(500.0);

        let backdrop = iced::widget::opaque(
            container(modal_box)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(Alignment::Center)
                .style(|theme: &iced::Theme| {
                    let overlay = AppTheme::from(theme).palette().overlay;
                    container::Style {
                        background: Some(overlay.backdrop_modal.into()),
                        ..Default::default()
                    }
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
