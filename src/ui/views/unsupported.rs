use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Theme};

use crate::app::Message;
use crate::app::messages::ActionMsg;
use crate::core::utils::{format_file_size_with_bytes, shorten_path};
use crate::features::common::parser::types::UnsupportedFileInfo;
use crate::ui::theme::tokens::{radius, spacing, typography};
use crate::ui::theme::{AppTheme, default_button_primary, default_button_secondary};

// Monochromatic SVG icons (stroke-based, clean vector)
static SVG_EXTERNAL_LINK: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>"#;

static SVG_FOLDER_OPEN: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path><path d="M2 10h20"></path></svg>"#;

static SVG_FILE_GENERIC: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>"#;

pub fn view_unsupported<'a>(
    info: &'a UnsupportedFileInfo,
    app_theme: AppTheme,
    font_family: Option<&'a str>,
    font_family_mono: Option<&'a str>,
) -> Element<'a, Message> {
    let palette = app_theme.palette();
    let base = palette.base;
    let roles = palette.roles;

    let main_font = crate::ui::theme::font::get_main_font(font_family);
    let mono_font = crate::ui::theme::font::get_code_font(font_family_mono);

    // Big Icon (48x48)
    let icon_name = crate::ui::theme::icon_theme::icon_for_entry(&info.file_name, false);
    let icon_element: Element<'a, Message> = if let Some(svg_handle) =
        crate::ui::theme::icon_theme::get_icon_handle(icon_name)
            .or_else(|| crate::ui::theme::icon_theme::get_icon_handle("application-octet-stream"))
            .or_else(|| crate::ui::theme::icon_theme::get_icon_handle("application-x-generic"))
    {
        svg(svg_handle)
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .into()
    } else {
        svg(svg::Handle::from_memory(SVG_FILE_GENERIC))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .style(move |_theme: &Theme, _status| svg::Style {
                color: Some(base.text_dim),
            })
            .into()
    };

    let icon_container = container(icon_element)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    // File name
    let file_name_text = text(&info.file_name)
        .size(typography::TITLE_SM)
        .font(main_font)
        .style(move |_| iced::widget::text::Style {
            color: Some(base.text),
        })
        .wrapping(text::Wrapping::None);

    // Badge status: "No preview available (binary format)" or "(unsupported format)"
    let badge_label = if info.is_binary {
        "No preview available (binary format)"
    } else {
        "No preview available (unsupported format)"
    };

    let badge_color = roles.danger;
    let badge_bg = Color {
        a: 0.15,
        ..badge_color
    };

    let badge = container(
        row![
            text("⚠")
                .size(typography::BADGE)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(badge_color),
                }),
            text(badge_label)
                .size(typography::CAPTION)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(badge_color),
                })
        ]
        .spacing(spacing::XS)
        .align_y(Alignment::Center),
    )
    .padding(Padding {
        top: 3.0,
        right: 10.0,
        bottom: 3.0,
        left: 10.0,
    })
    .style(move |_theme: &Theme| container::Style {
        background: Some(badge_bg.into()),
        border: Border {
            radius: radius::FLOATING_PILL.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    });

    // Metadata Grid Rows
    let formatted_size = format_file_size_with_bytes(info.file_size);

    let modified_str = info
        .modified
        .map(crate::core::utils::human_time)
        .unwrap_or_else(|| "Unknown".to_string());

    let shortened_location = shorten_path(&info.location, 36);

    let grid_content = column![
        info_row(
            "File Type",
            info.file_type.clone(),
            false,
            main_font,
            mono_font,
            base.text_dim,
            base.text
        ),
        info_row(
            "File Size",
            formatted_size,
            true, // use mono for technical/bytes display
            main_font,
            mono_font,
            base.text_dim,
            base.text
        ),
        info_row(
            "Last Modified",
            modified_str,
            false,
            main_font,
            mono_font,
            base.text_dim,
            base.text
        ),
        info_row(
            "Location",
            shortened_location,
            true,
            main_font,
            mono_font,
            base.text_dim,
            base.text
        ),
    ]
    .spacing(spacing::XS + 2.0)
    .width(Length::Fill);

    let grid_box = container(grid_content)
        .width(Length::Fill)
        .padding(Padding::from([spacing::S + 2.0, spacing::M]))
        .style(move |_theme: &Theme| container::Style {
            background: Some(base.surface_raised.into()),
            border: Border {
                radius: radius::LG.into(),
                width: 1.0,
                color: base.border,
            },
            ..Default::default()
        });

    // Vector Icon Buttons
    let open_icon = svg(svg::Handle::from_memory(SVG_EXTERNAL_LINK))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(move |_theme: &Theme, _status| svg::Style {
            color: Some(roles.accent_text),
        });

    let open_btn = button(
        row![
            open_icon,
            text("Open with App")
                .size(typography::BODY_MD)
                .font(main_font),
        ]
        .spacing(spacing::S)
        .align_y(Alignment::Center),
    )
    .on_press(ActionMsg::OpenClicked.into())
    .style(default_button_primary)
    .padding(Padding {
        top: 8.0,
        right: 14.0,
        bottom: 8.0,
        left: 14.0,
    })
    .width(Length::Fill);

    let folder_icon = svg(svg::Handle::from_memory(SVG_FOLDER_OPEN))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(move |_theme: &Theme, _status| svg::Style {
            color: Some(base.text),
        });

    let show_folder_btn = button(
        row![
            folder_icon,
            text("Show in Folder")
                .size(typography::BODY_MD)
                .font(main_font),
        ]
        .spacing(spacing::S)
        .align_y(Alignment::Center),
    )
    .on_press(ActionMsg::ShowInFolder.into())
    .style(default_button_secondary)
    .padding(Padding {
        top: 8.0,
        right: 14.0,
        bottom: 8.0,
        left: 14.0,
    })
    .width(Length::Fill);

    let action_row = row![open_btn, show_folder_btn]
        .spacing(spacing::M)
        .width(Length::Fill);

    let main_body = column![
        icon_container,
        file_name_text,
        badge,
        Space::new().height(Length::Fixed(spacing::XS)),
        grid_box,
        Space::new().height(Length::Fixed(spacing::XS)),
        action_row,
    ]
    .spacing(spacing::S)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(main_body)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: spacing::XL,
            right: spacing::XL,
            bottom: spacing::XL,
            left: spacing::XL,
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn info_row<'a>(
    label: &'static str,
    value: String,
    is_mono: bool,
    main_font: iced::Font,
    mono_font: iced::Font,
    label_color: Color,
    val_color: Color,
) -> Element<'a, Message> {
    let font = if is_mono { mono_font } else { main_font };

    row![
        container(
            text(label)
                .size(typography::CAPTION)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(label_color),
                })
        )
        .width(Length::Fixed(90.0)),
        container(
            text(value)
                .size(typography::CAPTION)
                .font(font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(val_color),
                })
                .wrapping(text::Wrapping::None)
        )
        .clip(true)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_unsupported_constructs_without_panic() {
        let info = UnsupportedFileInfo {
            file_name: "test_binary.pkt".to_string(),
            file_size: 1024,
            file_type: "Packet Capture / Octet-stream".to_string(),
            modified: Some(std::time::SystemTime::UNIX_EPOCH),
            location: "/tmp".to_string(),
            is_binary: true,
        };

        let _element = view_unsupported(&info, AppTheme::Dark, None, None);
    }
}
