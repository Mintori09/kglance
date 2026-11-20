use crate::app::Message;
use crate::app::messages::Model3dMsg;
use crate::core::types::KglanceState;
use crate::features::model3d::pipeline::model3d_widget;
use crate::features::model3d::types::Model3dMesh;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::{elevation, radius, spacing, typography};
use iced::widget::{button, container, row, stack, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};
use std::sync::Arc;

fn hud_pill_container(theme: &Theme) -> container::Style {
    let app_theme = AppTheme::from(theme);
    let overlay = &app_theme.palette().overlay;
    container::Style {
        background: Some(Background::Color(overlay.floating_bg)),
        text_color: Some(overlay.floating_fg),
        border: Border {
            color: overlay.floating_border,
            width: 1.0,
            radius: radius::FLOATING_PILL.into(),
        },
        shadow: elevation::floating_pill(overlay.floating_shadow),
        snap: false,
    }
}

fn ghost_btn_style(theme: &Theme, status: button::Status) -> button::Style {
    let app_theme = AppTheme::from(theme);
    let overlay = &app_theme.palette().overlay;

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
            overlay.floating_fg,
            overlay.floating_border,
        ),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(),
        },
        text_color,
        shadow: Shadow::default(),
        snap: false,
    }
}

fn toggle_btn_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let app_theme = AppTheme::from(theme);
        let role = &app_theme.palette().roles;
        let overlay = &app_theme.palette().overlay;

        let (bg, text_color, border_color) = if active {
            match status {
                button::Status::Hovered => (role.accent_hover, role.accent_text, role.accent_hover),
                button::Status::Pressed => {
                    (role.accent_pressed, role.accent_text, role.accent_pressed)
                }
                _ => (role.accent, role.accent_text, role.accent),
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
                    overlay.floating_text_dim,
                    overlay.floating_border,
                ),
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius::LG.into(),
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
    let reset_btn = iced::widget::tooltip(
        button(text("Reset").size(typography::BODY))
            .padding([4, 10])
            .style(ghost_btn_style)
            .on_press(Message::Model3d(Model3dMsg::ResetCamera)),
        text("Reset camera view (R)").size(typography::CAPTION),
        iced::widget::tooltip::Position::Top,
    );

    // 2. Shading mode cycle button
    let shading_label = format!("Shading: {}", state.model3d.shading);
    let shading_btn = iced::widget::tooltip(
        button(text(shading_label).size(typography::BODY))
            .padding([4, 10])
            .style(ghost_btn_style)
            .on_press(Message::Model3d(Model3dMsg::ToggleShading)),
        text("Toggle shading mode (S)").size(typography::CAPTION),
        iced::widget::tooltip::Position::Top,
    );

    // 3. Grid toggle button
    let grid_label = if state.model3d.show_grid {
        "Grid on"
    } else {
        "Grid off"
    };
    let grid_btn = iced::widget::tooltip(
        button(text(grid_label).size(typography::BODY))
            .padding([4, 10])
            .style(toggle_btn_style(state.model3d.show_grid))
            .on_press(Message::Model3d(Model3dMsg::ToggleGrid)),
        text("Toggle ground grid (G)").size(typography::CAPTION),
        iced::widget::tooltip::Position::Top,
    );

    // 4. Auto-rotate button (Accent when ON, ghost/dark when OFF)
    let auto_rotate_label = if state.model3d.auto_rotate {
        "Auto-rotate on"
    } else {
        "Auto-rotate off"
    };
    let auto_rotate_btn = iced::widget::tooltip(
        button(text(auto_rotate_label).size(typography::BODY))
            .padding([4, 10])
            .style(toggle_btn_style(state.model3d.auto_rotate))
            .on_press(Message::Model3d(Model3dMsg::ToggleAutoRotate)),
        text("Toggle continuous rotation (Space)").size(typography::CAPTION),
        iced::widget::tooltip::Position::Top,
    );

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
    let dim_label = format!("{:.1} × {:.1} × {:.1} units", size[0], size[1], size[2]);
    let info_text = text(dim_label)
        .size(typography::BODY)
        .style(|theme: &Theme| {
            let app_theme = AppTheme::from(theme);
            text::Style {
                color: Some(app_theme.palette().overlay.floating_text_dim),
            }
        });

    let controls = row![
        reset_btn,
        shading_btn,
        grid_btn,
        auto_rotate_btn,
        divider,
        info_text,
    ]
    .spacing(spacing::S)
    .align_y(Alignment::Center);

    let floating_pill = container(controls)
        .padding([spacing::S, spacing::M])
        .style(hud_pill_container);

    let overlay_controls = container(floating_pill)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(spacing::L)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(Alignment::End);

    // Studio background container: theme base surface/bg
    let studio_bg_viewport = container(viewport)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let app_theme = AppTheme::from(theme);
            let base = &app_theme.palette().base;
            container::Style {
                background: Some(Background::Color(base.surface)),
                ..Default::default()
            }
        });

    stack![studio_bg_viewport, overlay_controls]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
