use std::path::Path;

use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Shadow, Theme, alignment};

use crate::app::Message;
use crate::core::{SortField, TableState};
use crate::ui::theme::{DARK_TEXT, LIGHT_TEXT, glass_row_button, glass_scrollable, icon_theme};

const FONT_WEIGHT_BOLD: Font = Font {
    weight: Weight::Bold,
    ..Font::DEFAULT
};

const FONT_WEIGHT_MEDIUM: Font = Font {
    weight: Weight::Medium,
    ..Font::DEFAULT
};

const COLUMN_PORTION_NAME: u16 = 65;
const COLUMN_PORTION_KIND: u16 = 10;
const COLUMN_PORTION_SIZE: u16 = 10;
const COLUMN_PORTION_MODIFIED: u16 = 15;

const HOVER_OPACITY: f32 = 0.06;
const DEFAULT_OPACITY: f32 = 0.03;

pub fn view_table<'a>(state: &'a TableState, theme_dark: bool) -> Element<'a, Message> {
    let (text_color, dim_color, sub_dim_color) = resolve_theme_colors(theme_dark);

    let summary_block = create_summary_block(state, dim_color, sub_dim_color);
    let header = create_table_header(&state.sort_state);
    let rows_list = create_rows_list(state, text_color, dim_color);

    column![
        summary_block,
        header,
        scrollable(rows_list)
            .style(glass_scrollable)
            .height(Length::Fill)
    ]
    .spacing(4)
    .into()
}

fn resolve_theme_colors(theme_dark: bool) -> (Color, Color, Color) {
    let base = if theme_dark { DARK_TEXT } else { LIGHT_TEXT };

    (base, Color { a: 0.75, ..base }, Color { a: 0.50, ..base })
}

fn create_summary_block<'a>(
    state: &'a TableState,
    dim_color: Color,
    sub_dim_color: Color,
) -> Element<'a, Message> {
    let folder_name = Path::new(&state.folder_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Folder");

    let total_files = state.rows.iter().filter(|r| !r.is_dir).count();
    let total_dirs = state.rows.iter().filter(|r| r.is_dir).count();
    let human_total_size = crate::parsers::human_size(state.total_size);

    let stats_text = if total_dirs > 0 {
        format!("{total_files} files, {total_dirs} folders • {human_total_size}")
    } else {
        format!("{total_files} files • {human_total_size}")
    };

    let folder_icon: Element<'a, Message> =
        if let Some(handle) = icon_theme::get_icon_handle("inode-directory") {
            svg(handle).width(18).height(18).into()
        } else {
            text("📁").size(16).into()
        };

    column![
        row![
            folder_icon,
            text(folder_name).size(14).font(FONT_WEIGHT_BOLD),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text(&state.folder_path).size(11).color(sub_dim_color),
        text(stats_text).size(12).color(dim_color),
    ]
    .spacing(4)
    .padding([8, 12])
    .into()
}

fn create_table_header<'a>(sort: &crate::core::SortState) -> Element<'a, Message> {
    let create_header_button = |field: SortField, label: &str, portion: u16| {
        let sort_text = format_sort_label(sort, field, label);
        button(text(sort_text).size(12).font(FONT_WEIGHT_MEDIUM))
            .on_press(Message::SortByFieldClicked(field))
            .style(header_button_style)
            .width(Length::FillPortion(portion))
    };

    container(
        row![
            create_header_button(SortField::Name, "Name", COLUMN_PORTION_NAME),
            create_header_button(SortField::Kind, "Kind", COLUMN_PORTION_KIND),
            create_header_button(SortField::Size, "Size", COLUMN_PORTION_SIZE),
            create_header_button(SortField::Modified, "Modified", COLUMN_PORTION_MODIFIED),
        ]
        .spacing(10)
        .padding(4),
    )
    .padding([2, 4])
    .into()
}

fn format_sort_label(sort: &crate::core::SortState, field: SortField, label: &str) -> String {
    if sort.active && sort.field == field {
        let indicator = if sort.ascending { "▲" } else { "▼" };
        format!("{label} {indicator}")
    } else {
        label.to_string()
    }
}

fn header_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = matches!(theme, Theme::Dark);
    let text_color = if is_dark { DARK_TEXT } else { LIGHT_TEXT };

    let alpha = match status {
        button::Status::Hovered => HOVER_OPACITY,
        _ => DEFAULT_OPACITY,
    };

    let base_color = if is_dark { Color::WHITE } else { Color::BLACK };
    let background = Some(
        Color {
            a: alpha,
            ..base_color
        }
        .into(),
    );

    button::Style {
        background,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

fn create_rows_list<'a>(
    state: &'a TableState,
    text_color: Color,
    dim_color: Color,
) -> Element<'a, Message> {
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
            row![icon_el, text(&row_data.name).size(13).color(text_color)]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::FillPortion(COLUMN_PORTION_NAME)),
            text(&row_data.kind)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(COLUMN_PORTION_KIND)),
            text(&row_data.size)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(COLUMN_PORTION_SIZE))
                .align_x(alignment::Horizontal::Right),
            text(&row_data.modified)
                .size(12)
                .color(dim_color)
                .width(Length::FillPortion(COLUMN_PORTION_MODIFIED)),
        ]
        .align_y(Alignment::Center)
        .padding([4, 6])
        .spacing(10);

        let row_btn = button(row_content)
            .on_press(Message::FileClicked(idx))
            .style(move |theme, status| glass_row_button(theme, status, is_selected))
            .padding(0)
            .height(34);

        rows_list = rows_list.push(row_btn);
    }

    rows_list.into()
}
