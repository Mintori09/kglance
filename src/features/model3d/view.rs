use crate::app::Message;
use crate::app::messages::Model3dMsg;
use crate::core::types::KglanceState;
use crate::features::model3d::pipeline::model3d_widget;
use crate::features::model3d::types::Model3dMesh;
use crate::ui::theme::color::primitive;
use crate::ui::theme::tokens::radius;
use iced::widget::{button, container, row, stack, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};
use std::sync::Arc;

// Semi-transparent dark HUD pill to guarantee contrast on any background/model color
// Concentric border radius: Outer (14px) = Inner Button (8px) + Padding (6px)
fn hud_pill_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.10, 0.10, 0.13, 0.88))),
        text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: iced::Vector::new(0.0, 3.0),
            blur_radius: 8.0,
        },
        snap: false,
    }
}

fn ghost_btn_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Hovered => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.16),
            Color::WHITE,
            Color::from_rgba(1.0, 1.0, 1.0, 0.25),
        ),
        button::Status::Pressed => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.24),
            Color::WHITE,
            Color::from_rgba(1.0, 1.0, 1.0, 0.32),
        ),
        _ => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            Color::from_rgb(0.92, 0.92, 0.94),
            Color::from_rgba(1.0, 1.0, 1.0, 0.12),
        ),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(), // 8.0px concentric radius
        },
        text_color,
        shadow: Shadow::default(),
        snap: false,
    }
}

fn toggle_btn_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color, border_color) = if active {
            match status {
                button::Status::Hovered => (
                    primitive::ACCENT_HOVER,
                    Color::WHITE,
                    primitive::ACCENT_HOVER,
                ),
                button::Status::Pressed => (
                    primitive::ACCENT_PRESSED,
                    Color::WHITE,
                    primitive::ACCENT_PRESSED,
                ),
                _ => (primitive::ACCENT, Color::WHITE, primitive::ACCENT),
            }
        } else {
            match status {
                button::Status::Hovered => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.16),
                    Color::WHITE,
                    Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                ),
                button::Status::Pressed => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.24),
                    Color::WHITE,
                    Color::from_rgba(1.0, 1.0, 1.0, 0.32),
                ),
                _ => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    Color::from_rgb(0.88, 0.88, 0.90),
                    Color::from_rgba(1.0, 1.0, 1.0, 0.12),
                ),
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius::LG.into(), // 8.0px concentric radius
            },
            text_color,
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn view_model3d<'a>(state: &'a KglanceState, mesh: &'a Model3dMesh) -> Element<'a, Message> {
    let vertices_arc = Arc::clone(&mesh.vertices);
    let indices_arc = Arc::clone(&mesh.indices);

    let callbacks = crate::features::model3d::pipeline::Model3dCallbacks {
        on_drag_start: Box::new(|pt| Message::Model3d(Model3dMsg::DragStarted(pt))),
        on_drag_move: Box::new(|pt| Message::Model3d(Model3dMsg::DragMoved(pt))),
        on_drag_end: Message::Model3d(Model3dMsg::DragEnded),
        on_zoom: Box::new(|f| Message::Model3d(Model3dMsg::Zoom(f))),
    };

    let viewport = model3d_widget(
        &state.model3d,
        vertices_arc,
        indices_arc,
        mesh.bounds,
        callbacks,
    );

    // 1. Reset button (Ghost/secondary style)
    let reset_btn = button(text("Reset").size(12))
        .padding([4, 10])
        .style(ghost_btn_style)
        .on_press(Message::Model3d(Model3dMsg::ResetCamera));

    // 2. Shading mode cycle button
    let shading_label = format!("Shading: {}", state.model3d.shading);
    let shading_btn = button(text(shading_label).size(12))
        .padding([4, 10])
        .style(ghost_btn_style)
        .on_press(Message::Model3d(Model3dMsg::ToggleShading));

    // 3. Grid toggle button
    let grid_label = if state.model3d.show_grid {
        "Grid: ON"
    } else {
        "Grid: OFF"
    };
    let grid_btn = button(text(grid_label).size(12))
        .padding([4, 10])
        .style(toggle_btn_style(state.model3d.show_grid))
        .on_press(Message::Model3d(Model3dMsg::ToggleGrid));

    // 4. Auto-rotate button (Accent when ON, ghost/dark when OFF)
    let auto_rotate_label = if state.model3d.auto_rotate {
        "Auto-Rotate: ON"
    } else {
        "Auto-Rotate: OFF"
    };
    let auto_rotate_btn = button(text(auto_rotate_label).size(12))
        .padding([4, 10])
        .style(toggle_btn_style(state.model3d.auto_rotate))
        .on_press(Message::Model3d(Model3dMsg::ToggleAutoRotate));

    // Subtle vertical separator between controls and dimension label
    let divider =
        container(iced::widget::Space::new().width(1.0).height(16.0)).style(|_theme: &Theme| {
            container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.18))),
                ..Default::default()
            }
        });

    // 5. Bounding box dimension information with explicit units
    let size = mesh.bounds.size();
    let dim_label = format!("📐 {:.1} × {:.1} × {:.1} units", size[0], size[1], size[2]);
    let info_text = text(dim_label)
        .size(12)
        .color(iced::Color::from_rgba(0.75, 0.75, 0.75, 0.9));

    let controls = row![
        reset_btn,
        shading_btn,
        grid_btn,
        auto_rotate_btn,
        divider,
        info_text,
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let floating_pill = container(controls)
        .padding([6, 12])
        .style(hud_pill_container);

    let overlay_controls = container(floating_pill)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(18)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(Alignment::End);

    // Studio background container: subtly tinted #F4F6F9 in light theme / sleek #121214 in dark
    let studio_bg_viewport = container(viewport)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let is_dark = theme.palette().background.r < 0.5;
            let bg = if is_dark {
                Color::from_rgb8(18, 18, 20)
            } else {
                Color::from_rgb8(244, 246, 249)
            };
            container::Style {
                background: Some(Background::Color(bg)),
                ..Default::default()
            }
        });

    stack![studio_bg_viewport, overlay_controls]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
