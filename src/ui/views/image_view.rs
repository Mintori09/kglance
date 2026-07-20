use crate::app::Message;
use crate::core::ImageState;
use crate::ui::theme::{breeze_button, glass_card, glass_scrollable};
use iced::widget::canvas::{Frame, Geometry};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Rectangle, Renderer, Theme, mouse};

// =========================================================================
// 1. Domain / Geometry Helpers (SRP & DIP)
// =========================================================================

/// Đảm bảo tính toán tỷ lệ hiển thị ảnh (Aspect Fit) độc lập với UI Framework.
struct ImageBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ImageBounds {
    fn calculate(image_width: f32, image_height: f32, bounds: Rectangle) -> Option<Self> {
        if image_width <= 0.0 || image_height <= 0.0 {
            return None;
        }

        let scale = (bounds.width / image_width).min(bounds.height / image_height);
        let width = image_width * scale;
        let height = image_height * scale;
        let x = (bounds.width - width) / 2.0;
        let y = (bounds.height - height) / 2.0;

        Some(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Chuyển đổi từ tọa độ con trỏ chuột sang tọa độ chuẩn hóa (0.0..=1.0)
    fn normalize_point(&self, pos_x: f32, pos_y: f32) -> Option<(f32, f32)> {
        let nx = (pos_x - self.x) / self.width;
        let ny = (pos_y - self.y) / self.height;

        if (0.0..=1.0).contains(&nx) && (0.0..=1.0).contains(&ny) {
            Some((nx, ny))
        } else {
            None
        }
    }
}

// =========================================================================
// 2. Canvas Program Implementation
// =========================================================================

struct ImageCanvasProgram {
    image_bytes: Vec<u8>,
    image_width: u32,
    image_height: u32,
    picker_enabled: bool,
}

impl canvas::Program<Message> for ImageCanvasProgram {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if !self.picker_enabled {
            return (canvas::event::Status::Ignored, None);
        }

        let Some(pos) = cursor.position_in(bounds) else {
            return (canvas::event::Status::Ignored, None);
        };

        let Some(img_bounds) =
            ImageBounds::calculate(self.image_width as f32, self.image_height as f32, bounds)
        else {
            return (canvas::event::Status::Ignored, None);
        };

        if let Some((nx, ny)) = img_bounds.normalize_point(pos.x, pos.y) {
            return match event {
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => (
                    canvas::event::Status::Captured,
                    Some(Message::ImageHovered { x: nx, y: ny }),
                ),
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => (
                    canvas::event::Status::Captured,
                    Some(Message::ImageClicked { x: nx, y: ny }),
                ),
                _ => (canvas::event::Status::Ignored, None),
            };
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if let Some(img_bounds) =
            ImageBounds::calculate(self.image_width as f32, self.image_height as f32, bounds)
        {
            let handle = iced::widget::image::Handle::from_bytes(self.image_bytes.clone());
            frame.draw_image(
                Rectangle {
                    x: img_bounds.x,
                    y: img_bounds.y,
                    width: img_bounds.width,
                    height: img_bounds.height,
                },
                &handle,
            );
        }

        vec![frame.into_geometry()]
    }
}

// =========================================================================
// 3. UI Component Builders (Single Responsibility)
// =========================================================================

/// Dựng swatch hiển thị màu sắc
fn create_color_swatch<'a>(color_opt: Option<(u8, u8, u8)>) -> Element<'a, Message> {
    if let Some((r, g, b)) = color_opt {
        container(text(""))
            .width(20)
            .height(20)
            .style(move |_| container::Style {
                background: Some(Color::from_rgb8(r, g, b).into()),
                border: iced::Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        container(text("")).into()
    }
}

/// Dựng thanh xem trước màu cho Color Picker
fn build_color_picker_preview<'a>(state: &'a ImageState) -> Element<'a, Message> {
    let hover_swatch = create_color_swatch(state.cursor_color);
    let picked_swatch = create_color_swatch(state.picked_color);

    row![
        text("Hover:"),
        hover_swatch,
        text(&state.cursor_color_hex).size(12),
        text("Picked:"),
        picked_swatch,
        text(&state.picked_color_hex).size(12),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

/// Dựng thanh công cụ Toolbar phía trên
fn build_toolbar<'a>(state: &'a ImageState) -> Element<'a, Message> {
    let picker_btn_text = if state.picker_enabled {
        "Disable Picker"
    } else {
        "Color Picker"
    };

    let mut toolbar = row![
        button(text("Zoom In"))
            .on_press(Message::ImageZoomIn)
            .style(breeze_button),
        button(text("Zoom Out"))
            .on_press(Message::ImageZoomOut)
            .style(breeze_button),
        button(text("Rotate L"))
            .on_press(Message::ImageRotateLeft)
            .style(breeze_button),
        button(text("Rotate R"))
            .on_press(Message::ImageRotateRight)
            .style(breeze_button),
        button(text("Reset"))
            .on_press(Message::ImageReset)
            .style(breeze_button),
        button(text(picker_btn_text))
            .on_press(Message::ToggleColorPicker)
            .style(breeze_button),
        button(text("Exif Info"))
            .on_press(Message::ToggleExifSidebar)
            .style(breeze_button),
    ]
    .spacing(10)
    .padding(5)
    .align_y(Alignment::Center);

    if state.picker_enabled {
        toolbar = toolbar.push(build_color_picker_preview(state));
    }

    toolbar.into()
}

/// Dựng thành phần hiển thị nội dung Ảnh (SVG hoặc Canvas)
fn build_image_viewport<'a>(
    state: &'a ImageState,
    image_bytes: &'a [u8],
    is_svg: bool,
) -> Element<'a, Message> {
    let (iw, ih) = probe_image_dimensions(image_bytes);

    let content: Element<'a, Message> = if is_svg {
        let handle = iced::widget::svg::Handle::from_memory(image_bytes.to_vec());
        iced::widget::svg(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        canvas(ImageCanvasProgram {
            image_bytes: image_bytes.to_vec(),
            image_width: iw,
            image_height: ih,
            picker_enabled: state.picker_enabled,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                ..Default::default()
            }
        })
        .into()
}

/// Dựng Sidebar hiển thị EXIF
fn build_exif_sidebar<'a>(exif_content: &'a str) -> Element<'a, Message> {
    container(scrollable(text(exif_content).size(14)).style(glass_scrollable))
        .width(Length::FillPortion(1))
        .padding(10)
        .style(glass_card)
        .into()
}

/// Utility lấy kích thước ảnh từ memory
fn probe_image_dimensions(bytes: &[u8]) -> (u32, u32) {
    if let Ok(img) = image::load_from_memory(bytes) {
        use image::GenericImageView;
        img.dimensions()
    } else {
        (640, 480)
    }
}

// =========================================================================
// 4. Main Entry View Function
// =========================================================================

pub fn view_image<'a>(
    state: &'a ImageState,
    image_bytes: &'a [u8],
    is_svg: bool,
) -> Element<'a, Message> {
    let toolbar = build_toolbar(state);
    let viewport = build_image_viewport(state, image_bytes, is_svg);
    let left_column = column![toolbar, viewport].width(Length::Fill);

    if state.show_exif {
        let sidebar = build_exif_sidebar(&state.exif_content);
        row![left_column.width(Length::FillPortion(4)), sidebar]
            .spacing(5)
            .into()
    } else {
        row![left_column].into()
    }
}
