use std::path::Path;

use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Shadow, Theme, alignment};

use crate::app::Message;
use crate::core::{FolderState, SortField};
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::color::primitive;
use crate::ui::theme::{default_row_button, default_scrollable, icon_theme};

use crate::ui::theme::tokens::{spacing, tables};

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

const HEADER_HOVER_OPACITY: f32 = 0.06;
const HEADER_DEFAULT_OPACITY: f32 = 0.03;

const SORT_ASCENDING_INDICATOR: &str = "▲";
const SORT_DESCENDING_INDICATOR: &str = "▼";

const DEFAULT_FOLDER_NAME: &str = "Folder";
const DEFAULT_FOLDER_EMOJI: &str = "📁";
const FOLDER_ICON_NAME: &str = "inode-directory";

const SUMMARY_FOLDER_NAME_SIZE: f32 = 14.0;
const SUMMARY_PATH_SIZE: f32 = 10.0;
const SUMMARY_STATS_SIZE: f32 = 12.0;
const SUMMARY_ICON_SIZE: f32 = 18.0;

const HEADER_TEXT_SIZE: f32 = tables::FONT_SIZE_BODY;
const ROW_TEXT_SIZE: f32 = tables::FONT_SIZE_BODY;
const ROW_NAME_SIZE: f32 = 14.0;
const ROW_ICON_SIZE: f32 = spacing::L;

const MAIN_LAYOUT_SPACING: f32 = spacing::XS;
const SUMMARY_LAYOUT_SPACING: f32 = spacing::XS;
const SUMMARY_TITLE_SPACING: f32 = spacing::S;
const HEADER_LAYOUT_SPACING: f32 = spacing::S;
const ROW_CONTENT_SPACING: f32 = spacing::S;
const ROW_NAME_SPACING: f32 = spacing::S;
const ROWS_LIST_SPACING: f32 = spacing::XXS;

const ROW_BUTTON_HEIGHT: f32 = tables::ROW_HEIGHT;

const SUMMARY_PADDING: [u16; 2] = [spacing::S as u16, spacing::M as u16];
const HEADER_CONTAINER_PADDING: [u16; 2] = [spacing::XXS as u16, spacing::XS as u16];
const HEADER_ROW_PADDING: u16 = spacing::XS as u16;
const ROW_CONTENT_PADDING: [u16; 2] = [spacing::XS as u16, spacing::S as u16];

pub fn view_folder<'a>(state: &'a FolderState, theme_dark: bool) -> Element<'a, Message> {
    let (text_color, dim_color, sub_dim_color) = resolve_theme_colors(theme_dark);

    let summary_block = create_summary_block(state, dim_color, sub_dim_color);
    let folder_header = create_folder_header(&state.sort_state);
    let rows_list = create_folder_rows(state, text_color, dim_color);

    column![
        summary_block,
        folder_header,
        scrollable(rows_list)
            .style(default_scrollable)
            .height(Length::Fill)
    ]
    .spacing(MAIN_LAYOUT_SPACING)
    .into()
}

fn resolve_theme_colors(theme_dark: bool) -> (Color, Color, Color) {
    let base_color = BaseColors::palette_for(theme_dark).text;

    let text_color = base_color;
    let dim_color = Color {
        a: 0.75,
        ..base_color
    };
    let sub_dim_color = Color {
        a: 0.50,
        ..base_color
    };

    (text_color, dim_color, sub_dim_color)
}

fn create_summary_block<'a>(
    state: &'a FolderState,
    dim_color: Color,
    sub_dim_color: Color,
) -> Element<'a, Message> {
    let folder_name = Path::new(&state.folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_FOLDER_NAME);

    let total_files = state.rows.iter().filter(|row| !row.is_dir).count();
    let total_dirs = state.rows.iter().filter(|row| row.is_dir).count();
    let human_total_size = crate::parsers::human_size(state.total_size);

    let stats_text = if total_dirs > 0 {
        format!("{total_files} files, {total_dirs} folders • {human_total_size}")
    } else {
        format!("{total_files} files • {human_total_size}")
    };

    let folder_icon: Element<'a, Message> =
        if let Some(svg_handle) = icon_theme::get_icon_handle(FOLDER_ICON_NAME) {
            svg(svg_handle)
                .width(SUMMARY_ICON_SIZE)
                .height(SUMMARY_ICON_SIZE)
                .into()
        } else {
            text(DEFAULT_FOLDER_EMOJI).size(SUMMARY_ICON_SIZE).into()
        };

    column![
        row![
            folder_icon,
            text(folder_name)
                .size(SUMMARY_FOLDER_NAME_SIZE)
                .font(FONT_WEIGHT_BOLD),
        ]
        .spacing(SUMMARY_TITLE_SPACING)
        .align_y(Alignment::Center),
        text(&state.folder_path)
            .size(SUMMARY_PATH_SIZE)
            .color(sub_dim_color),
        text(stats_text).size(SUMMARY_STATS_SIZE).color(dim_color),
    ]
    .spacing(SUMMARY_LAYOUT_SPACING)
    .padding(SUMMARY_PADDING)
    .into()
}

fn create_folder_header<'a>(sort_state: &crate::core::SortState) -> Element<'a, Message> {
    container(
        row![
            create_header_button(sort_state, SortField::Name, "Name", COLUMN_PORTION_NAME),
            create_header_button(sort_state, SortField::Kind, "Kind", COLUMN_PORTION_KIND),
            create_header_button(sort_state, SortField::Size, "Size", COLUMN_PORTION_SIZE),
            create_header_button(
                sort_state,
                SortField::Modified,
                "Modified",
                COLUMN_PORTION_MODIFIED
            ),
        ]
        .spacing(HEADER_LAYOUT_SPACING)
        .padding(HEADER_ROW_PADDING),
    )
    .padding(HEADER_CONTAINER_PADDING)
    .into()
}

fn create_header_button<'a>(
    sort_state: &crate::core::SortState,
    field: SortField,
    label: &str,
    width_portion: u16,
) -> button::Button<'a, Message> {
    let sort_text = format_sort_label(sort_state, field, label);

    button(
        text(sort_text)
            .size(HEADER_TEXT_SIZE)
            .font(FONT_WEIGHT_MEDIUM),
    )
    .on_press(Message::SortByFieldClicked(field))
    .style(header_button_style)
    .width(Length::FillPortion(width_portion))
}

fn format_sort_label(sort_state: &crate::core::SortState, field: SortField, label: &str) -> String {
    if sort_state.active && sort_state.field == field {
        let indicator = if sort_state.ascending {
            SORT_ASCENDING_INDICATOR
        } else {
            SORT_DESCENDING_INDICATOR
        };
        format!("{label} {indicator}")
    } else {
        label.to_string()
    }
}

fn header_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = BaseColors::palette(theme);

    let hover_alpha = match status {
        button::Status::Hovered => HEADER_HOVER_OPACITY,
        _ => HEADER_DEFAULT_OPACITY,
    };

    let is_light_theme = palette.bg.r > 0.5;
    let base_color = if is_light_theme {
        primitive::BLACK
    } else {
        primitive::WHITE
    };

    let background_color = Color {
        a: hover_alpha,
        ..base_color
    };

    button::Style {
        background: Some(background_color.into()),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

fn create_folder_rows<'a>(
    state: &'a FolderState,
    text_color: Color,
    dim_color: Color,
) -> Element<'a, Message> {
    let mut rows_list = column![].spacing(ROWS_LIST_SPACING);

    for (row_index, row_data) in state.rows.iter().enumerate() {
        let is_selected = state.selected_index == Some(row_index);
        let row_button = create_folder_row(row_index, row_data, is_selected, text_color, dim_color);
        rows_list = rows_list.push(row_button);
    }

    rows_list.into()
}

fn create_folder_row<'a>(
    row_index: usize,
    row_data: &'a crate::core::FolderRowState,
    is_selected: bool,
    text_color: Color,
    dim_color: Color,
) -> button::Button<'a, Message> {
    let icon_element = render_row_icon(row_data.icon);

    let row_content = row![
        row![
            icon_element,
            text(&row_data.name).size(ROW_NAME_SIZE).color(text_color)
        ]
        .spacing(ROW_NAME_SPACING)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(COLUMN_PORTION_NAME)),
        text(&row_data.kind)
            .size(ROW_TEXT_SIZE)
            .color(dim_color)
            .width(Length::FillPortion(COLUMN_PORTION_KIND)),
        text(&row_data.size)
            .size(ROW_TEXT_SIZE)
            .color(dim_color)
            .width(Length::FillPortion(COLUMN_PORTION_SIZE))
            .align_x(alignment::Horizontal::Right),
        text(&row_data.modified)
            .size(ROW_TEXT_SIZE)
            .color(dim_color)
            .width(Length::FillPortion(COLUMN_PORTION_MODIFIED)),
    ]
    .align_y(Alignment::Center)
    .padding(ROW_CONTENT_PADDING)
    .spacing(ROW_CONTENT_SPACING);

    button(row_content)
        .on_press(Message::FileClicked(row_index))
        .style(move |theme, status| default_row_button(theme, status, is_selected))
        .padding(0)
        .height(ROW_BUTTON_HEIGHT)
}

fn render_row_icon<'a>(icon_name: &str) -> Element<'a, Message> {
    if let Some(svg_handle) = icon_theme::get_icon_handle(icon_name) {
        svg(svg_handle)
            .width(ROW_ICON_SIZE)
            .height(ROW_ICON_SIZE)
            .into()
    } else {
        text(" ").size(ROW_ICON_SIZE).into()
    }
}
