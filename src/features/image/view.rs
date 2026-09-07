use crate::app::Message;
use crate::core::ImageState;
use crate::features::image::ImageCanvas;
use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget::{container, text};

const EMPTY_LABEL: &str = "No image loaded";
const EMPTY_LABEL_FONT_SIZE: f32 = 18.0;
const HEADER_HEIGHT: f32 = 50.0;

pub fn is_loaded(state: &ImageState) -> bool {
    state.handle.is_some() || state.preview_handle.is_some() || state.display_handle.is_some()
}

pub fn view_image<'a>(state: &'a ImageState, font_family: Option<&str>) -> Element<'a, Message> {
    let canvas = render_image(state);

    if state.show_info && !state.exif_content.is_empty() {
        let card = render_info_card(state, font_family);
        iced::widget::Stack::new()
            .push(canvas)
            .push(
                container(card)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(16)
                    .align_x(iced::alignment::Horizontal::Right)
                    .align_y(iced::alignment::Vertical::Top),
            )
            .into()
    } else {
        canvas
    }
}

fn render_info_card<'a>(state: &'a ImageState, font_family: Option<&str>) -> Element<'a, Message> {
    use iced::widget::{button, column, row};
    use iced::{Border, Padding, Shadow, Vector};

    let font = crate::ui::theme::font::get_main_font(font_family);

    let title_row = row![
        text("Image Info")
            .size(13)
            .font(font)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                iced::widget::text::Style {
                    color: Some(palette.primary.base.color),
                }
            }),
        iced::widget::Space::new().width(Length::Fill),
        button(text("✕").size(11).font(font))
            .on_press(crate::app::messages::ImageMsg::CloseInfo.into())
            .style(iced::widget::button::text)
            .padding([0, 4]),
    ]
    .align_y(iced::Alignment::Center);

    let mut lines_col = column![title_row].spacing(6);

    for line in state.exif_content.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let row = row![
                text(format!("{}:", k.trim()))
                    .size(11)
                    .font(font)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::text::Style {
                            color: Some(palette.background.weak.text),
                        }
                    }),
                text(v.trim())
                    .size(11)
                    .font(font)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::text::Style {
                            color: Some(palette.background.base.text),
                        }
                    }),
            ]
            .spacing(6);
            lines_col = lines_col.push(row);
        } else {
            lines_col = lines_col.push(text(line).size(11).font(font));
        }
    }

    container(lines_col)
        .padding(Padding {
            top: 10.0,
            right: 14.0,
            bottom: 12.0,
            left: 14.0,
        })
        .max_width(320.0)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            let mut bg = palette.background.base.color;
            bg.a = 0.92;
            container::Style {
                background: Some(bg.into()),
                text_color: Some(palette.background.base.text),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                shadow: Shadow {
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 10.0,
                    color: iced::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.35,
                    },
                },
                ..Default::default()
            }
        })
        .into()
}

pub fn calculate_window_size(
    window_w: f32,
    window_h: f32,
    img_width: u32,
    img_height: u32,
) -> Size {
    let win_w = if window_w > 0.0 { window_w } else { 1024.0 };
    let win_h = if window_h > 0.0 { window_h } else { 768.0 };

    if img_width == 0 || img_height == 0 {
        return Size::new(win_w, win_h);
    }

    let img_w = img_width as f32;
    let img_h = img_height as f32;

    let scale = (win_w / img_w).min(win_h / img_h).min(1.0);

    let calculated_w = (img_w * scale).max(200.0);
    let calculated_h = (img_h * scale + HEADER_HEIGHT).max(150.0);

    Size::new(calculated_w, calculated_h)
}

pub fn render_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    ImageCanvas::new(state, &state.camera)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_zoom(|factor, cursor| crate::app::messages::ImageMsg::Zoom { factor, cursor }.into())
        .on_drag(|dx, dy| crate::app::messages::ImageMsg::PanDelta(dx, dy).into())
        .on_double_click(|| crate::app::messages::ImageMsg::FitToWindow.into())
        .into()
}

pub fn render_placeholder<'a>() -> Element<'a, Message> {
    container(text(EMPTY_LABEL).size(EMPTY_LABEL_FONT_SIZE))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_window_size_with_zero_dimensions() {
        let size = calculate_window_size(0.0, 0.0, 1920, 1080);
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);

        let size_zero_img = calculate_window_size(800.0, 600.0, 0, 0);
        assert_eq!(size_zero_img.width, 800.0);
        assert_eq!(size_zero_img.height, 600.0);
    }

    #[test]
    fn test_calculate_window_size_scaling() {
        let size = calculate_window_size(1000.0, 800.0, 500, 400);
        assert_eq!(size.width, 500.0);
        assert_eq!(size.height, 400.0 + HEADER_HEIGHT);
    }

    #[test]
    fn test_view_image_with_and_without_info() {
        let mut state = ImageState {
            exif_content: "Camera Make: Nikon\nCamera Model: Z6".to_string(),
            show_info: false,
            ..Default::default()
        };
        let _ = view_image(&state, None);

        state.show_info = true;
        let _ = view_image(&state, Some("Noto Sans"));
    }
}
