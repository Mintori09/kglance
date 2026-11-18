use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Border, Element, Length, Shadow};

use crate::app::Message;
use crate::app::messages::EpubMsg;
use crate::core::types::EpubState;
use crate::ui::components::button::breeze_close;
use crate::ui::theme::tokens::{border, radius, spacing, typography};
use crate::ui::theme::{AppTheme, default_tooltip};

pub fn render_lightbox<'a>(state: &'a EpubState) -> Element<'a, Message> {
    let Some(lightbox) = &state.lightbox else {
        return container(text("")).width(0).height(0).into();
    };

    let total = lightbox.all_image_paths.len().max(1);
    let current = lightbox.current_index + 1;
    let title_text = format!("Image {current} / {total}");

    // Header bar
    let title_label = text(title_text)
        .size(typography::BODY_MD)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(AppTheme::from(theme).palette().overlay.floating_fg),
        });

    let filename = std::path::Path::new(&lightbox.active_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&lightbox.active_path);

    let filename_label = text(filename)
        .size(typography::CAPTION)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(AppTheme::from(theme).palette().overlay.floating_text_dim),
        });

    let close_btn = tooltip(
        button(text("✕").size(typography::BODY_LG))
            .on_press(EpubMsg::CloseImageLightbox.into())
            .padding([spacing::XS, spacing::S])
            .style(breeze_close),
        text("Close").size(typography::CAPTION),
        tooltip::Position::Bottom,
    )
    .style(default_tooltip);

    let top_bar = container(
        row![
            row![title_label, filename_label]
                .spacing(spacing::M)
                .align_y(Alignment::Center),
            iced::widget::Space::new().width(Length::Fill),
            close_btn,
        ]
        .align_y(Alignment::Center)
        .padding([spacing::S, spacing::L])
        .width(Length::Fill),
    )
    .style(|theme: &iced::Theme| container::Style {
        background: Some(AppTheme::from(theme).palette().overlay.floating_bg.into()),
        border: Border {
            width: border::NONE,
            color: iced::Color::TRANSPARENT,
            radius: radius::NONE.into(),
        },
        ..Default::default()
    });

    // Image view canvas
    let image_canvas = crate::features::image::view::render_image(&lightbox.image_state);

    // Prev / Next buttons
    let has_multiple = lightbox.all_image_paths.len() > 1;
    let prev_btn: Element<'a, Message> = if has_multiple {
        tooltip(
            button(text("‹").size(typography::HERO))
                .on_press(EpubMsg::PrevLightboxImage.into())
                .padding([spacing::S, spacing::M])
                .style(nav_button_style),
            text("Previous image").size(typography::CAPTION),
            tooltip::Position::Right,
        )
        .style(default_tooltip)
        .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let next_btn: Element<'a, Message> = if has_multiple {
        tooltip(
            button(text("›").size(typography::HERO))
                .on_press(EpubMsg::NextLightboxImage.into())
                .padding([spacing::S, spacing::M])
                .style(nav_button_style),
            text("Next image").size(typography::CAPTION),
            tooltip::Position::Left,
        )
        .style(default_tooltip)
        .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let main_view = row![
        container(prev_btn)
            .width(Length::Shrink)
            .height(Length::Fill)
            .align_y(Alignment::Center)
            .padding([spacing::NONE, spacing::M]),
        container(image_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        container(next_btn)
            .width(Length::Shrink)
            .height(Length::Fill)
            .align_y(Alignment::Center)
            .padding([spacing::NONE, spacing::M]),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Center);

    let full_content = iced::widget::column![top_bar, main_view]
        .width(Length::Fill)
        .height(Length::Fill);

    container(full_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(
                AppTheme::from(theme)
                    .palette()
                    .overlay
                    .backdrop_modal
                    .into(),
            ),
            ..Default::default()
        })
        .into()
}

fn nav_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let overlay = AppTheme::from(theme).palette().overlay;
    let mut hover_bg = overlay.floating_bg;
    let mut press_bg = overlay.floating_bg;
    hover_bg.a = 0.85;
    press_bg.a = 0.95;
    let bg = match status {
        button::Status::Hovered => Some(hover_bg.into()),
        button::Status::Pressed => Some(press_bg.into()),
        _ => Some(overlay.floating_bg.into()),
    };
    button::Style {
        background: bg,
        text_color: overlay.floating_fg,
        border: Border {
            radius: radius::FULL.into(),
            width: border::THIN,
            color: overlay.floating_border,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
