use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Border, Element, Length, Shadow};

use super::types::LightboxState;
use crate::app::Message;
use crate::app::messages::{LightboxMsg, SystemMsg};
use crate::ui::components::button::breeze_close;
use crate::ui::theme::tokens::{border, radius, spacing, typography};
use crate::ui::theme::{AppTheme, default_tooltip};

pub fn render_lightbox<'a>(lightbox: &'a LightboxState) -> Element<'a, Message> {
    if lightbox.items.is_empty() {
        return container(text("")).width(0).height(0).into();
    }

    let total = lightbox.items.len().max(1);
    let current = lightbox.current_index + 1;
    let item = &lightbox.items[lightbox.current_index.min(total - 1)];

    let index_text = format!("Image {current} / {total}");
    let index_label = text(index_text)
        .size(typography::BODY_MD)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(AppTheme::from(theme).palette().overlay.floating_fg),
        });

    let subtitle_label =
        text(&item.subtitle)
            .size(typography::CAPTION)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(AppTheme::from(theme).palette().overlay.floating_text_dim),
            });

    let mut info_row = row![index_label]
        .spacing(spacing::S)
        .align_y(Alignment::Center);

    if let Some(ref title) = item.title {
        let title_label = text(title)
            .size(typography::BODY)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(AppTheme::from(theme).palette().overlay.floating_fg),
            });
        info_row = info_row.push(title_label);
    }
    info_row = info_row.push(subtitle_label);

    let link_btn: Option<Element<'a, Message>> = item.link_url.as_ref().map(|url| {
        let url_clone = url.clone();
        button(
            row![
                text("Open link").size(typography::CAPTION),
                text("↗").size(typography::CAPTION)
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center),
        )
        .on_press(SystemMsg::OpenLink(url_clone).into())
        .padding([spacing::XS, spacing::S])
        .style(|theme: &iced::Theme, status: button::Status| {
            let palette = AppTheme::from(theme).palette();
            let mut hover_bg = palette.roles.accent;
            let mut press_bg = palette.roles.accent;
            let mut base_bg = palette.roles.accent;
            let mut border_col = palette.roles.accent;
            hover_bg.a = 0.30;
            press_bg.a = 0.50;
            base_bg.a = 0.15;
            border_col.a = 0.35;
            let bg = match status {
                button::Status::Hovered => Some(hover_bg.into()),
                button::Status::Pressed => Some(press_bg.into()),
                _ => Some(base_bg.into()),
            };
            button::Style {
                background: bg,
                text_color: palette.overlay.floating_fg,
                border: Border {
                    radius: radius::SM.into(),
                    width: border::THIN,
                    color: border_col,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
    });

    let close_btn = tooltip(
        button(text("✕").size(typography::BODY_LG))
            .on_press(LightboxMsg::Close.into())
            .padding([spacing::XS, spacing::S])
            .style(breeze_close),
        text("Close").size(typography::CAPTION),
        tooltip::Position::Bottom,
    )
    .style(default_tooltip);

    let mut right_actions = row![].spacing(spacing::S).align_y(Alignment::Center);
    if let Some(link) = link_btn {
        right_actions = right_actions.push(link);
    }
    right_actions = right_actions.push(close_btn);

    let top_bar = container(
        row![
            info_row,
            iced::widget::Space::new().width(Length::Fill),
            right_actions,
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
    let has_multiple = lightbox.items.len() > 1;
    let prev_btn: Element<'a, Message> = if has_multiple {
        tooltip(
            button(text("‹").size(typography::HERO))
                .on_press(LightboxMsg::Prev.into())
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
                .on_press(LightboxMsg::Next.into())
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
