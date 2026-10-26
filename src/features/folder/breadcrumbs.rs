use std::path::Path;

use iced::widget::{button, row, text};
use iced::{Alignment, Border, Color, Element, Font, Shadow, Theme};

use crate::app::Message;
use crate::ui::theme::AppTheme;
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::tokens::{radius, spacing};

pub fn create_breadcrumbs<'a>(
    folder_path: &'a str,
    main_font: Font,
    theme: AppTheme,
) -> Element<'a, Message> {
    let path = Path::new(folder_path);
    let mut segments = Vec::new();

    let mut current = Some(path);
    while let Some(p) = current {
        if let Some(file_name) = p.file_name().and_then(|s| s.to_str()) {
            segments.push((file_name.to_string(), p.to_string_lossy().into_owned()));
        } else if p == Path::new("/") {
            segments.push(("/".to_string(), "/".to_string()));
            break;
        }
        current = p.parent();
    }
    segments.reverse();

    if segments.is_empty() {
        return text(folder_path).size(12.0).font(main_font).into();
    }

    let mut breadcrumbs_row = row![].spacing(spacing::XXS).align_y(Alignment::Center);
    let total = segments.len();

    for (idx, (name, target_path)) in segments.into_iter().enumerate() {
        let is_last = idx == total - 1;

        if is_last {
            let label = text(name)
                .size(11.5)
                .font(main_font)
                .color(theme.palette().base.text_dim);
            breadcrumbs_row = breadcrumbs_row.push(label);
        } else {
            let mut sub_dim = theme.palette().base.text_dim;
            sub_dim.a *= 0.75;
            let btn = button(text(name).size(11.5).font(main_font).color(sub_dim))
                .padding([2, 4])
                .on_press(crate::app::messages::NavigationMsg::OpenPath(target_path).into())
                .style(breadcrumb_button_style);

            let separator = text("›").size(11.5).font(main_font).color(sub_dim);

            breadcrumbs_row = breadcrumbs_row.push(btn).push(separator);
        }
    }

    breadcrumbs_row.into()
}

fn breadcrumb_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = crate::ui::theme::color::roles::palette(theme);
    let mut sub_dim = p.text_dim;
    sub_dim.a *= 0.75;

    let (bg, text_color) = match status {
        button::Status::Hovered => (Some(Color { a: 0.10, ..p.text }.into()), role.accent_hover),
        button::Status::Pressed => (
            Some(Color { a: 0.18, ..p.text }.into()),
            role.accent_pressed,
        ),
        _ => (None, sub_dim),
    };

    button::Style {
        background: bg,
        text_color,
        border: Border {
            radius: radius::XS.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
