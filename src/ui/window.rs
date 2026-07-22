use iced::{
    Alignment, Element, Length, Padding,
    widget::{Stack, column, container, row, text},
};

use crate::app::Message;
use crate::core::KglanceState;

fn metadata_text(state: &KglanceState) -> String {
    let mut parts = Vec::new();

    if let Some(folder_name) = std::path::Path::new(&state.file_name)
        .parent()
        .and_then(|p| p.file_name())
    {
        let name = folder_name.to_string_lossy();
        if !name.is_empty() && name != "/" {
            parts.push(format!("{}/", name));
        }
    }

    if !state.file_type_text.is_empty() {
        parts.push(state.file_type_text.clone());
    }

    if !state.file_size_text.is_empty() {
        parts.push(state.file_size_text.clone());
    }

    if !state.file_modified_text.is_empty() {
        parts.push(state.file_modified_text.clone());
    }

    parts.join(" • ")
}

fn footer<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    let meta = metadata_text(state);
    if meta.is_empty() {
        return Element::from(container(text("")).padding(0));
    }

    container(
        row![text(meta).size(11).style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::text::Style {
                color: Some(palette.background.weak.text),
            }
        })]
        .align_y(Alignment::Center)
        .padding([4, 12]),
    )
    .width(Length::Fill)
    .style(crate::ui::theme::breeze_header_container)
    .into()
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
                            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
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
    let layout = column![content(preview_body, edge_to_edge), footer(state),]
        .width(Length::Fill)
        .height(Length::Fill);

    let base = container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::theme::breeze_container);

    let toast_layer = container(toasts(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End)
        .align_x(iced::alignment::Horizontal::Center);

    Stack::new().push(base).push(toast_layer).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_text_full() {
        let state = KglanceState {
            file_name: "/home/mintori/Desktop/subfolder/test_doc.md".to_string(),
            file_type_text: "Markdown Document".to_string(),
            file_size_text: "12.4 KB".to_string(),
            file_modified_text: "2026-07-22".to_string(),
            ..Default::default()
        };

        let meta = metadata_text(&state);
        assert_eq!(
            meta,
            "subfolder/ • Markdown Document • 12.4 KB • 2026-07-22"
        );
    }

    #[test]
    fn test_metadata_text_partial() {
        let state = KglanceState {
            file_name: "test_doc.md".to_string(),
            file_type_text: "Text Document".to_string(),
            ..Default::default()
        };

        let meta = metadata_text(&state);
        assert_eq!(meta, "Text Document");
    }
}
