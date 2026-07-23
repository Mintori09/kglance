use crate::app::Message;
use crate::core::types::GridThumbnail;
use crate::ui::theme::icon_theme;
use iced::widget::{button, column, container, image, responsive, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Shadow, Theme};
use std::path::{Path, PathBuf};

fn truncate_middle(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let half = (max_chars.saturating_sub(1)) / 2;
    let start: String = s.chars().take(half).collect();
    let end: String = s.chars().rev().take(half).collect::<String>().chars().rev().collect();
    format!("{}…{}", start, end)
}

pub fn get_freedesktop_thumbnail_path(file_path: &str) -> Option<PathBuf> {

    let canonical = Path::new(file_path).canonicalize().ok()?;
    let uri = format!("file://{}", canonical.display());
    let digest = md5::compute(uri.as_bytes());
    let hex_hash = format!("{:x}.png", digest);

    let home = std::env::var("HOME").ok()?;
    let large = PathBuf::from(&home)
        .join(".cache/thumbnails/large")
        .join(&hex_hash);
    if large.exists() {
        return Some(large);
    }
    let normal = PathBuf::from(&home)
        .join(".cache/thumbnails/normal")
        .join(&hex_hash);
    if normal.exists() {
        return Some(normal);
    }
    None
}

pub fn view_grid<'a>(thumbnails: &'a [GridThumbnail], active_index: usize) -> Element<'a, Message> {
    responsive(move |bounds| {
        let item_width = 150.0;
        let gap = 12.0;
        let cols = ((bounds.width - gap) / (item_width + gap)).floor().max(1.0) as usize;

        let mut col_widget = column![].spacing(gap).padding(gap);

        for (chunk_idx, chunk) in thumbnails.chunks(cols).enumerate() {
            let mut row_widget = row![].spacing(gap);
            for (item_idx, item) in chunk.iter().enumerate() {
                let global_idx = chunk_idx * cols + item_idx;
                let is_active = global_idx == active_index;

                let card_content: Element<'_, Message> =
                    if let Some(ref handle) = item.thumbnail_handle {
                        container(
                            image(handle.clone())
                                .width(130.0)
                                .height(90.0)
                                .content_fit(iced::ContentFit::Contain),
                        )
                        .width(136.0)
                        .height(96.0)
                        .center_x(136.0)
                        .center_y(96.0)
                        .into()
                    } else {
                        let icon_name = icon_theme::icon_for_entry(&item.name, false);
                        let icon_el: Element<'_, Message> =
                            if let Some(svg_handle) = icon_theme::get_icon_handle(icon_name) {
                                svg(svg_handle).width(48.0).height(48.0).into()
                            } else {
                                text("📄").size(36).into()
                            };

                        container(icon_el)
                            .width(136.0)
                            .height(96.0)
                            .center_x(136.0)
                            .center_y(96.0)
                            .into()
                    };

                let card_btn = button(
                    column![
                        card_content,
                        text(truncate_middle(&item.name, 20))
                            .size(11)
                            .shaping(text::Shaping::Advanced)
                            .width(Length::Fill)
                            .align_x(Alignment::Center)

                    ]
                    .align_x(Alignment::Center)
                    .spacing(4)
                    .padding(6),
                )
                .on_press(Message::FileClickedInGrid(global_idx))
                .style(move |theme: &Theme, status: button::Status| {
                    let is_dark = matches!(theme, Theme::Dark);

                    let accent_blue = Color::from_rgb(0.24, 0.68, 0.91);

                    let (bg, border_color, border_width, shadow) = if is_active {
                        let active_bg = if is_dark {
                            Color::from_rgba(0.24, 0.68, 0.91, 0.25)
                        } else {
                            Color::from_rgba(0.24, 0.68, 0.91, 0.20)
                        };
                        let glow_shadow = Shadow {
                            color: Color::from_rgba(0.24, 0.68, 0.91, 0.4),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 8.0,
                        };
                        (active_bg, accent_blue, 2.0, glow_shadow)
                    } else {
                        match status {
                            button::Status::Hovered => {
                                let hover_bg = if is_dark {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.12)
                                } else {
                                    Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                                };
                                let hover_border = if is_dark {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                                } else {
                                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                                };
                                (hover_bg, hover_border, 1.0, Shadow::default())
                            }
                            _ => {
                                let base_bg = if is_dark {
                                    Color::from_rgba(0.14, 0.16, 0.20, 0.6)
                                } else {
                                    Color::from_rgba(0.98, 0.98, 1.0, 0.7)
                                };
                                let base_border = if is_dark {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                                } else {
                                    Color::from_rgba(0.0, 0.0, 0.0, 0.07)
                                };
                                (base_bg, base_border, 1.0, Shadow::default())
                            }
                        }
                    };

                    button::Style {
                        background: Some(bg.into()),
                        text_color: if is_dark { Color::WHITE } else { Color::BLACK },
                        border: Border {
                            color: border_color,
                            width: border_width,
                            radius: 10.0.into(),
                        },
                        shadow,
                        snap: false,
                    }
                })
                .width(item_width);

                row_widget = row_widget.push(card_btn);
            }
            col_widget = col_widget.push(row_widget);
        }

        scrollable(col_widget)
            .id("grid_scroll")
            .height(Length::Fill)
            .width(Length::Fill)
            .into()

    })
    .into()
}
