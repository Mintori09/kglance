use crate::app::Message;
use crate::core::ImageState;
use crate::features::image::ImageCanvas;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::{elevation, radius, spacing, typography};
use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget::{container, text};

const EMPTY_LABEL: &str = "No image loaded";
const EMPTY_LABEL_FONT_SIZE: f32 = typography::DISPLAY;
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
                    .padding(spacing::L)
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
    use iced::{Border, Padding};

    let font = crate::ui::theme::font::get_main_font(font_family);

    let title_row = row![
        text("Image Info")
            .size(typography::BODY_MD)
            .font(font)
            .style(|theme: &iced::Theme| {
                let role = AppTheme::from(theme).palette().roles;
                iced::widget::text::Style {
                    color: Some(role.accent),
                }
            }),
        iced::widget::Space::new().width(Length::Fill),
        button(text("✕").size(typography::CAPTION).font(font))
            .on_press(crate::app::messages::ImageMsg::CloseInfo.into())
            .style(iced::widget::button::text)
            .padding([0, 4]),
    ]
    .align_y(iced::Alignment::Center);

    let mut lines_col = column![title_row].spacing(spacing::XS + 2.0);

    for (k, v) in &state.exif_parsed_lines {
        let row = row![
            text(format!("{}:", k))
                .size(typography::CAPTION)
                .font(font)
                .style(|theme: &iced::Theme| {
                    let p = AppTheme::from(theme).palette().base;
                    iced::widget::text::Style {
                        color: Some(p.text_dim),
                    }
                }),
            text(v.as_str())
                .size(typography::CAPTION)
                .font(font)
                .style(|theme: &iced::Theme| {
                    let p = AppTheme::from(theme).palette().base;
                    iced::widget::text::Style {
                        color: Some(p.text),
                    }
                }),
        ]
        .spacing(spacing::XS + 2.0);
        lines_col = lines_col.push(row);
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
            let palette = AppTheme::from(theme).palette();
            let mut bg = palette.base.surface;
            bg.a = 0.92;
            container::Style {
                background: Some(bg.into()),
                text_color: Some(palette.base.text),
                border: Border {
                    radius: radius::LG.into(),
                    width: 1.0,
                    color: palette.base.border,
                },
                shadow: elevation::floating_pill(palette.base.shadow),
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
