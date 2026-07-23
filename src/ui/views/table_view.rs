use crate::app::Message;
use crate::core::{SortField, TableState};
use crate::ui::theme::glass_scrollable;
use crate::ui::theme::icon_theme;
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Border, Color, Element, Length, Shadow};

pub fn view_table<'a>(state: &'a TableState, theme_dark: bool) -> Element<'a, Message> {
    // Determine dynamic theme colors
    let text_color = if theme_dark {
        Color::from_rgb(0.93, 0.94, 0.96)
    } else {
        Color::from_rgb(0.12, 0.13, 0.16)
    };

    let dim_color = if theme_dark {
        Color::from_rgba(0.93, 0.94, 0.96, 0.75)
    } else {
        Color::from_rgba(0.12, 0.13, 0.16, 0.75)
    };

    let sub_dim_color = if theme_dark {
        Color::from_rgba(0.93, 0.94, 0.96, 0.50)
    } else {
        Color::from_rgba(0.12, 0.13, 0.16, 0.50)
    };

    // 1. Folder Summary Block
    let folder_name = std::path::Path::new(&state.folder_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Folder");

    let total_files = state.rows.iter().filter(|r| !r.is_dir).count();
    let total_dirs = state.rows.iter().filter(|r| r.is_dir).count();
    let human_total_size = crate::parsers::human_size(state.total_size);

    let stats_text = if total_dirs > 0 {
        format!(
            "{} files, {} folders • {}",
            total_files, total_dirs, human_total_size
        )
    } else {
        format!("{} files • {}", total_files, human_total_size)
    };

    let folder_icon: Element<'a, Message> =
        if let Some(handle) = icon_theme::get_icon_handle("inode-directory") {
            svg(handle).width(18).height(18).into()
        } else {
            text("📁").size(16).into()
        };

    let summary_block = column![
        row![
            folder_icon,
            text(folder_name).size(14).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text(&state.folder_path).size(11).color(sub_dim_color),
        text(stats_text).size(12).color(dim_color),
    ]
    .spacing(4)
    .padding([8, 12]);

    // Helper for header sort button styles (Finder-like header buttons)
    let header_button_style = move |theme: &iced::Theme, status: button::Status| -> button::Style {
        let is_dark = matches!(theme, iced::Theme::Dark);
        let t_color = if is_dark {
            Color::from_rgb(0.93, 0.94, 0.96)
        } else {
            Color::from_rgb(0.12, 0.13, 0.16)
        };
        let bg = match status {
            button::Status::Hovered => {
                let mut c = if is_dark { Color::WHITE } else { Color::BLACK };
                c.a = 0.06;
                Some(c.into())
            }
            _ => {
                let mut c = if is_dark { Color::WHITE } else { Color::BLACK };
                c.a = 0.03;
                Some(c.into())
            }
        };
        button::Style {
            background: bg,
            text_color: t_color,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        }
    };

    // 2. Simplified Table Header with sort indicators
    let sort = &state.sort_state;
    let sort_label = |field: SortField, label: &str| -> String {
        if sort.active && sort.field == field {
            if sort.ascending {
                format!("{label} ▲")
            } else {
                format!("{label} ▼")
            }
        } else {
            label.to_string()
        }
    };

    let header = container(
        row![
            button(
                text(sort_label(SortField::Name, "Name"))
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
            )
            .on_press(Message::SortByFieldClicked(SortField::Name))
            .style(header_button_style)
            .width(Length::FillPortion(65)),
            button(
                text(sort_label(SortField::Kind, "Kind"))
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
            )
            .on_press(Message::SortByFieldClicked(SortField::Kind))
            .style(header_button_style)
            .width(Length::FillPortion(10)),
            button(
                text(sort_label(SortField::Size, "Size"))
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
            )
            .on_press(Message::SortByFieldClicked(SortField::Size))
            .style(header_button_style)
            .width(Length::FillPortion(10)),
            button(
                text(sort_label(SortField::Modified, "Modified"))
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
            )
            .on_press(Message::SortByFieldClicked(SortField::Modified))
            .style(header_button_style)
            .width(Length::FillPortion(15)),
        ]
        .spacing(10)
        .padding(4),
    )
    .padding([2, 4]);

    // 3. Rows List
    let mut rows_list = column![].spacing(2);
    for (idx, row_data) in state.rows.iter().enumerate() {
        let is_selected = state.selected_index == Some(idx);

        let icon_el: Element<'a, Message> =
            if let Some(handle) = icon_theme::get_icon_handle(row_data.icon) {
                svg(handle).width(16).height(16).into()
            } else {
                text("  ").size(16).into()
            };

        let row_content = row![
            row![icon_el, text(&row_data.name).size(13).color(text_color),]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .width(Length::FillPortion(65)),
            text(&row_data.kind)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(10)),
            text(&row_data.size)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(10))
                .align_x(iced::alignment::Horizontal::Right),
            text(&row_data.modified)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(15)),
        ]
        .align_y(iced::Alignment::Center)
        .padding([4, 6])
        .spacing(10);

        let row_btn = button(row_content)
            .on_press(Message::FileClicked(idx))
            .style(move |theme, status| {
                crate::ui::theme::glass_row_button(theme, status, is_selected)
            })
            .padding(0)
            .height(34);

        rows_list = rows_list.push(row_btn);
    }

    let main_view = column![
        summary_block,
        header,
        scrollable(rows_list)
            .style(glass_scrollable)
            .height(Length::Fill)
    ]
    .spacing(4);

    main_view.into()
}
