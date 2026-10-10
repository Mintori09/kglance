use std::path::Path;

use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Shadow, Theme, alignment};

use crate::app::Message;
use crate::core::{FolderState, SortField};
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::color::primitive;
use crate::ui::theme::font::{get_main_font, get_main_font_bold, get_main_font_medium};
use crate::ui::theme::tokens::{elevation, radius, spacing, tables, typography};
use crate::ui::theme::{AppTheme, default_scrollable, icon_theme};

const COLUMN_PORTION_NAME: u16 = 48;
const COLUMN_PORTION_KIND: u16 = 20;
const COLUMN_PORTION_SIZE: u16 = 14;
const COLUMN_PORTION_MODIFIED: u16 = 18;

const SORT_ASCENDING_INDICATOR: &str = "▲";
const SORT_DESCENDING_INDICATOR: &str = "▼";

const DEFAULT_FOLDER_NAME: &str = "Folder";
const DEFAULT_FOLDER_EMOJI: &str = "📁";
const FOLDER_ICON_NAME: &str = "inode-directory";

const SUMMARY_FOLDER_NAME_SIZE: f32 = 16.0;
const SUMMARY_PATH_SIZE: f32 = typography::CAPTION;
const SUMMARY_STATS_SIZE: f32 = 11.5;
const SUMMARY_ICON_SIZE: f32 = 26.0;

const HEADER_TEXT_SIZE: f32 = typography::BODY;
const ROW_TEXT_SIZE: f32 = typography::BODY;
const ROW_NAME_SIZE: f32 = 13.0;
const ROW_ICON_SIZE: f32 = spacing::L;

const VIEW_CONTAINER_PADDING: [u16; 2] = [spacing::S as u16, spacing::M as u16];
const MAIN_LAYOUT_SPACING: f32 = spacing::S;
const SUMMARY_TITLE_SPACING: f32 = spacing::M;
const HEADER_LAYOUT_SPACING: f32 = spacing::S;
const ROW_CONTENT_SPACING: f32 = spacing::S;
const ROW_NAME_SPACING: f32 = spacing::S;
const ROWS_LIST_SPACING: f32 = spacing::XXS;

const ROW_BUTTON_HEIGHT: f32 = tables::ROW_HEIGHT;

const SUMMARY_PADDING: [u16; 2] = [spacing::M as u16, spacing::L as u16];
const HEADER_CONTAINER_PADDING: [u16; 2] = [spacing::XS as u16, spacing::S as u16];
const HEADER_ROW_PADDING: u16 = spacing::XS as u16;
const ROW_CONTENT_PADDING: [u16; 2] = [spacing::XS as u16, spacing::S as u16];
const ROWS_LIST_PADDING: [u16; 2] = [spacing::XXS as u16, spacing::XXS as u16];

#[derive(Clone, Copy)]
struct FolderStyleContext {
    text_color: Color,
    dim_color: Color,
    sub_dim_color: Color,
    main_font: Font,
    main_font_bold: Font,
    main_font_medium: Font,
}

pub fn view_folder<'a>(
    state: &'a FolderState,
    theme: AppTheme,
    font_family: Option<&str>,
) -> Element<'a, Message> {
    let (text_color, dim_color, sub_dim_color) = resolve_theme_colors(theme);
    let style_ctx = FolderStyleContext {
        text_color,
        dim_color,
        sub_dim_color,
        main_font: get_main_font(font_family),
        main_font_bold: get_main_font_bold(font_family),
        main_font_medium: get_main_font_medium(font_family),
    };

    let summary_block = create_summary_block(state, &style_ctx);
    let folder_header = create_folder_header(&state.sort_state, style_ctx.main_font_medium);
    let rows_list = create_folder_rows(state, &style_ctx);

    let table_box = container(
        column![
            folder_header,
            scrollable(rows_list)
                .id("content_scroll")
                .style(default_scrollable)
                .on_scroll(|vp| crate::app::messages::NavigationMsg::FolderScrolled(vp).into())
                .height(Length::Fill)
        ]
        .spacing(spacing::XS)
        .padding(spacing::XS as u16),
    )
    .style(table_card_style)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![summary_block, table_box]
            .spacing(MAIN_LAYOUT_SPACING)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(VIEW_CONTAINER_PADDING)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn resolve_theme_colors(theme: AppTheme) -> (Color, Color, Color) {
    let base_color = theme.palette().base.text;

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
    ctx: &FolderStyleContext,
) -> Element<'a, Message> {
    let folder_name = Path::new(&state.folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_FOLDER_NAME);

    let total_files = state.rows.iter().filter(|row| !row.is_dir).count();
    let total_dirs = state.rows.iter().filter(|row| row.is_dir).count();
    let human_total_size = crate::parsers::human_size(state.total_size);

    let folder_icon: Element<'a, Message> =
        if let Some(svg_handle) = icon_theme::get_icon_handle(FOLDER_ICON_NAME) {
            svg(svg_handle)
                .width(SUMMARY_ICON_SIZE)
                .height(SUMMARY_ICON_SIZE)
                .into()
        } else {
            text(DEFAULT_FOLDER_EMOJI)
                .size(SUMMARY_ICON_SIZE)
                .font(ctx.main_font)
                .into()
        };

    let icon_box = container(folder_icon)
        .padding(spacing::S as u16)
        .style(summary_icon_box_style);

    let folder_info = column![
        text(folder_name)
            .size(SUMMARY_FOLDER_NAME_SIZE)
            .font(ctx.main_font_bold),
        text(&state.folder_path)
            .size(SUMMARY_PATH_SIZE)
            .font(ctx.main_font)
            .color(ctx.sub_dim_color),
    ]
    .spacing(spacing::XXS);

    let left_side = row![icon_box, folder_info]
        .spacing(SUMMARY_TITLE_SPACING)
        .align_y(Alignment::Center);

    let mut badges = row![].spacing(spacing::XS).align_y(Alignment::Center);

    let files_badge = container(
        text(format!("{total_files} files"))
            .size(SUMMARY_STATS_SIZE)
            .font(ctx.main_font_medium)
            .color(ctx.dim_color),
    )
    .padding([spacing::XS as u16, spacing::S as u16])
    .style(summary_badge_style);
    badges = badges.push(files_badge);

    if total_dirs > 0 {
        let dirs_badge = container(
            text(format!("{total_dirs} folders"))
                .size(SUMMARY_STATS_SIZE)
                .font(ctx.main_font_medium)
                .color(ctx.dim_color),
        )
        .padding([spacing::XS as u16, spacing::S as u16])
        .style(summary_badge_style);
        badges = badges.push(dirs_badge);
    }

    let size_badge = container(
        text(format!("{human_total_size} total"))
            .size(SUMMARY_STATS_SIZE)
            .font(ctx.main_font_medium)
            .color(ctx.dim_color),
    )
    .padding([spacing::XS as u16, spacing::S as u16])
    .style(summary_badge_style);
    badges = badges.push(size_badge);

    container(
        row![left_side, badges]
            .spacing(spacing::M)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(SUMMARY_PADDING)
    .style(summary_card_style)
    .width(Length::Fill)
    .into()
}

fn create_folder_header<'a>(
    sort_state: &crate::core::SortState,
    main_font_medium: Font,
) -> Element<'a, Message> {
    container(
        row![
            create_header_button(
                sort_state,
                SortField::Name,
                "Name",
                COLUMN_PORTION_NAME,
                main_font_medium,
                alignment::Horizontal::Left,
            ),
            create_header_button(
                sort_state,
                SortField::Kind,
                "Kind",
                COLUMN_PORTION_KIND,
                main_font_medium,
                alignment::Horizontal::Left,
            ),
            create_header_button(
                sort_state,
                SortField::Size,
                "Size",
                COLUMN_PORTION_SIZE,
                main_font_medium,
                alignment::Horizontal::Right,
            ),
            create_header_button(
                sort_state,
                SortField::Modified,
                "Modified",
                COLUMN_PORTION_MODIFIED,
                main_font_medium,
                alignment::Horizontal::Left,
            ),
        ]
        .spacing(HEADER_LAYOUT_SPACING)
        .align_y(Alignment::Center)
        .padding(HEADER_ROW_PADDING),
    )
    .padding(HEADER_CONTAINER_PADDING)
    .style(header_container_style)
    .width(Length::Fill)
    .into()
}

fn create_header_button<'a>(
    sort_state: &crate::core::SortState,
    field: SortField,
    label: &str,
    width_portion: u16,
    main_font_medium: Font,
    align: alignment::Horizontal,
) -> button::Button<'a, Message> {
    let sort_text = format_sort_label(sort_state, field, label);
    let is_active = sort_state.active && sort_state.field == field;

    button(
        text(sort_text)
            .size(HEADER_TEXT_SIZE)
            .font(main_font_medium)
            .align_x(align)
            .width(Length::Fill),
    )
    .on_press(Message::SortByFieldClicked(field))
    .style(move |theme, status| header_button_style(theme, status, is_active))
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

fn summary_card_style(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface_raised.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: radius::XL.into(),
        },
        shadow: elevation::low(p.shadow),
        snap: false,
    }
}

fn summary_icon_box_style(theme: &Theme) -> container::Style {
    let role = crate::ui::theme::color::roles::palette(theme);
    let mut bg = role.accent;
    bg.a = 0.15;
    let mut border_color = role.accent;
    border_color.a = 0.35;

    container::Style {
        background: Some(bg.into()),
        text_color: Some(role.accent),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn summary_badge_style(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text_dim),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: radius::MD.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn table_card_style(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: radius::XL.into(),
        },
        shadow: elevation::low(p.shadow),
        snap: false,
    }
}

fn header_container_style(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface_raised.into()),
        text_color: Some(p.text_dim),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: radius::MD.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn header_button_style(theme: &Theme, status: button::Status, is_active: bool) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = crate::ui::theme::color::roles::palette(theme);
    let app_theme = AppTheme::from(theme);

    let hover_bg = if app_theme.is_dark() {
        primitive::WHITE_006
    } else {
        primitive::BLACK_006
    };

    let (bg, text_color) = match status {
        button::Status::Hovered => (Some(hover_bg.into()), p.text),
        button::Status::Pressed => (Some(hover_bg.into()), role.accent_pressed),
        _ => {
            if is_active {
                (None, role.accent_hover)
            } else {
                (None, p.text_dim)
            }
        }
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

fn create_folder_rows<'a>(
    state: &'a FolderState,
    ctx: &FolderStyleContext,
) -> Element<'a, Message> {
    let mut rows_list = column![]
        .spacing(ROWS_LIST_SPACING)
        .padding(ROWS_LIST_PADDING);

    for (row_index, row_data) in state.rows.iter().enumerate() {
        let is_selected = state.selected_index == Some(row_index);
        let is_odd = row_index % 2 == 1;
        let row_button = create_folder_row(row_index, row_data, is_selected, is_odd, ctx);
        rows_list = rows_list.push(row_button);
    }

    rows_list.into()
}

fn create_folder_row<'a>(
    row_index: usize,
    row_data: &'a crate::core::FolderRowState,
    is_selected: bool,
    is_odd: bool,
    ctx: &FolderStyleContext,
) -> button::Button<'a, Message> {
    let icon_element = render_row_icon(row_data.icon, row_data.is_dir);

    let name_font = if row_data.is_dir {
        ctx.main_font_medium
    } else {
        ctx.main_font
    };

    let size_color = if row_data.size == "—" {
        ctx.sub_dim_color
    } else {
        ctx.dim_color
    };

    let row_content = row![
        row![
            icon_element,
            text(&row_data.name)
                .size(ROW_NAME_SIZE)
                .font(name_font)
                .color(ctx.text_color)
        ]
        .spacing(ROW_NAME_SPACING)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(COLUMN_PORTION_NAME)),
        text(&row_data.kind)
            .size(ROW_TEXT_SIZE)
            .font(ctx.main_font)
            .color(ctx.dim_color)
            .width(Length::FillPortion(COLUMN_PORTION_KIND)),
        text(&row_data.size)
            .size(ROW_TEXT_SIZE)
            .font(ctx.main_font)
            .color(size_color)
            .width(Length::FillPortion(COLUMN_PORTION_SIZE))
            .align_x(alignment::Horizontal::Right),
        text(&row_data.modified)
            .size(ROW_TEXT_SIZE)
            .font(ctx.main_font)
            .color(ctx.sub_dim_color)
            .width(Length::FillPortion(COLUMN_PORTION_MODIFIED)),
    ]
    .align_y(Alignment::Center)
    .padding(ROW_CONTENT_PADDING)
    .spacing(ROW_CONTENT_SPACING);

    button(row_content)
        .on_press(crate::app::messages::NavigationMsg::FileClicked(row_index).into())
        .style(move |theme, status| folder_row_style(theme, status, is_selected, is_odd))
        .padding(0)
        .height(ROW_BUTTON_HEIGHT)
}

fn folder_row_style(
    theme: &Theme,
    status: button::Status,
    is_selected: bool,
    is_odd: bool,
) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = crate::ui::theme::color::roles::palette(theme);
    let app_theme = AppTheme::from(theme);

    let zebra_bg = if app_theme.is_dark() {
        primitive::WHITE_002
    } else {
        primitive::BLACK_002
    };

    let hover_bg = if app_theme.is_dark() {
        primitive::WHITE_006
    } else {
        primitive::BLACK_006
    };

    let bg_color = match (is_selected, status) {
        (true, button::Status::Hovered) => {
            let mut c = role.accent;
            c.a = 0.22;
            Some(c.into())
        }
        (true, _) => {
            let mut c = role.accent;
            c.a = 0.16;
            Some(c.into())
        }
        (false, button::Status::Hovered) => Some(hover_bg.into()),
        (false, button::Status::Pressed) => Some(hover_bg.into()),
        (false, _) => {
            if is_odd {
                Some(zebra_bg.into())
            } else {
                None
            }
        }
    };

    let border = if is_selected {
        let mut bc = role.accent;
        bc.a = 0.40;
        Border {
            color: bc,
            width: 1.0,
            radius: radius::MD.into(),
        }
    } else {
        Border {
            radius: radius::MD.into(),
            ..Border::default()
        }
    };

    button::Style {
        background: bg_color,
        text_color: p.text,
        border,
        shadow: Shadow::default(),
        snap: false,
    }
}

fn render_row_icon<'a>(icon_name: &str, is_dir: bool) -> Element<'a, Message> {
    if let Some(svg_handle) = icon_theme::get_icon_handle(icon_name).or_else(|| {
        if is_dir {
            icon_theme::get_icon_handle("folder")
        } else {
            icon_theme::get_icon_handle("text-x-generic")
                .or_else(|| icon_theme::get_icon_handle("application-x-generic"))
        }
    }) {
        svg(svg_handle)
            .width(ROW_ICON_SIZE)
            .height(ROW_ICON_SIZE)
            .into()
    } else {
        let fallback = if is_dir { "📁" } else { "📄" };
        text(fallback).size(ROW_ICON_SIZE).into()
    }
}
