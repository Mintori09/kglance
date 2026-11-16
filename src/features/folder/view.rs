use std::path::Path;

use iced::widget::{Space, button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Shadow, Theme, alignment};

use super::breadcrumbs::create_breadcrumbs;
use super::distribution::create_distribution_bar;
use crate::app::Message;
use crate::core::{FolderState, SortField};
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::color::primitive;
use crate::ui::theme::font::{get_main_font, get_main_font_bold, get_main_font_medium};
use crate::ui::theme::tokens::{elevation, radius, spacing, tables, typography};
use crate::ui::theme::{AppTheme, default_scrollable, icon_theme};

const COLUMN_WIDTH_KIND: f32 = 140.0;
const COLUMN_WIDTH_SIZE: f32 = 80.0;
const COLUMN_WIDTH_MODIFIED: f32 = 130.0;
const SCROLLBAR_OFFSET: f32 = 12.0;

const SORT_ASCENDING_INDICATOR: &str = "▲";
const SORT_DESCENDING_INDICATOR: &str = "▼";

const DEFAULT_FOLDER_NAME: &str = "Folder";
const DEFAULT_FOLDER_EMOJI: &str = "📁";
const FOLDER_ICON_NAME: &str = "inode-directory";

const SUMMARY_FOLDER_NAME_SIZE: f32 = 16.0;
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
pub const FOLDER_ROW_TOTAL_HEIGHT: f32 = ROW_BUTTON_HEIGHT + ROWS_LIST_SPACING;
const OVERSCAN_ROWS: usize = 20;

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

    let summary_block = create_summary_block(state, &style_ctx, theme);
    let folder_header = create_folder_header(&state.sort_state, style_ctx.main_font_medium);
    let rows_list = create_folder_rows(state, &style_ctx);

    let mut table_content = column![folder_header].spacing(spacing::XS);

    if state.search_visible {
        table_content =
            table_content.push(search_bar(SearchKind::Folder, &state.search_query, None));
    }

    table_content = table_content.push(
        scrollable(rows_list)
            .id("content_scroll")
            .style(default_scrollable)
            .on_scroll(|vp| crate::app::messages::NavigationMsg::FolderScrolled(vp).into())
            .width(Length::Fill)
            .height(Length::Fill),
    );

    let table_box = container(table_content.padding(spacing::XS as u16))
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
    theme: AppTheme,
) -> Element<'a, Message> {
    let folder_name = Path::new(&state.folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_FOLDER_NAME);

    let (total_files, total_dirs) = state.rows.iter().fold((0, 0), |(files, dirs), row| {
        if row.is_dir {
            (files, dirs + 1)
        } else {
            (files + 1, dirs)
        }
    });
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

    let breadcrumbs = create_breadcrumbs(&state.folder_path, ctx.main_font, theme);

    let folder_info = column![
        text(folder_name)
            .size(SUMMARY_FOLDER_NAME_SIZE)
            .font(ctx.main_font_bold),
        breadcrumbs,
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

    let top_row = row![left_side, badges]
        .spacing(spacing::M)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let mut summary_content = column![top_row].spacing(spacing::S).width(Length::Fill);

    if let Some(dist_bar) = create_distribution_bar(&state.rows, theme) {
        summary_content = summary_content.push(dist_bar);
    }

    container(summary_content)
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
                Length::Fill,
                main_font_medium,
                alignment::Horizontal::Left,
            ),
            create_header_button(
                sort_state,
                SortField::Kind,
                "Kind",
                Length::Fixed(COLUMN_WIDTH_KIND),
                main_font_medium,
                alignment::Horizontal::Left,
            ),
            create_header_button(
                sort_state,
                SortField::Size,
                "Size",
                Length::Fixed(COLUMN_WIDTH_SIZE),
                main_font_medium,
                alignment::Horizontal::Right,
            ),
            create_header_button(
                sort_state,
                SortField::Modified,
                "Modified",
                Length::Fixed(COLUMN_WIDTH_MODIFIED),
                main_font_medium,
                alignment::Horizontal::Left,
            ),
            iced::widget::Space::new().width(Length::Fixed(SCROLLBAR_OFFSET)),
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
    width: Length,
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
    .width(width)
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
        primitive::WHITE_008
    } else {
        primitive::BLACK_008
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
    let visible_indices = super::tree::visible_folder_row_indices(state);
    let total_visible = visible_indices.len();

    let scroll_y = state.scroll_y.max(0.0);
    let raw_first = (scroll_y / FOLDER_ROW_TOTAL_HEIGHT).floor() as usize;
    let first_idx = raw_first.saturating_sub(OVERSCAN_ROWS).min(total_visible);

    let vh = if state.viewport_height > 0.0 {
        state.viewport_height
    } else {
        800.0
    };
    let visible_rows = (vh / FOLDER_ROW_TOTAL_HEIGHT).ceil() as usize;
    let visible_count = visible_rows + OVERSCAN_ROWS * 2;
    let last_idx = (first_idx + visible_count).min(total_visible);

    let window_indices = if first_idx < total_visible {
        &visible_indices[first_idx..last_idx]
    } else {
        &[]
    };

    let mut rows_list = column![]
        .spacing(ROWS_LIST_SPACING)
        .padding(ROWS_LIST_PADDING)
        .width(Length::Fill);

    if first_idx > 0 {
        let top_spacer = (first_idx as f32 * FOLDER_ROW_TOTAL_HEIGHT - ROWS_LIST_SPACING).max(0.0);
        rows_list = rows_list.push(Space::new().height(top_spacer));
    }

    for (loop_i, &row_index) in window_indices.iter().enumerate() {
        let vis_idx = first_idx + loop_i;
        let row_data = &state.rows[row_index];
        let is_selected = state.selected_index == Some(row_index);
        let is_odd = vis_idx % 2 == 1;
        let is_expanded = row_data.is_dir && state.expanded_paths.contains(&row_data.path);
        let row_button =
            create_folder_row(row_index, row_data, is_selected, is_odd, is_expanded, ctx);
        rows_list = rows_list.push(row_button);
    }

    if last_idx < total_visible {
        let remaining = total_visible - last_idx;
        let bottom_spacer =
            (remaining as f32 * FOLDER_ROW_TOTAL_HEIGHT - ROWS_LIST_SPACING).max(0.0);
        rows_list = rows_list.push(Space::new().height(bottom_spacer));
    }

    rows_list.into()
}

fn create_folder_row<'a>(
    row_index: usize,
    row_data: &'a crate::core::FolderRowState,
    is_selected: bool,
    is_odd: bool,
    is_expanded: bool,
    ctx: &FolderStyleContext,
) -> button::Button<'a, Message> {
    let icon_element = render_row_icon(row_data.icon, row_data.is_dir, is_expanded);

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

    let indent_width = (row_data.depth as f32) * 16.0;

    let chevron_element: Element<'a, Message> = if row_data.is_dir {
        let chevron_bytes = if is_expanded {
            crate::features::folder::tree::CHEVRON_DOWN_SVG
        } else {
            crate::features::folder::tree::CHEVRON_RIGHT_SVG
        };
        svg(svg::Handle::from_memory(chevron_bytes))
            .width(12.0)
            .height(12.0)
            .into()
    } else {
        Space::new().width(Length::Fixed(12.0)).into()
    };

    let row_content = row![
        row![
            if indent_width > 0.0 {
                Element::from(Space::new().width(Length::Fixed(indent_width)))
            } else {
                Element::from(Space::new().width(Length::Fixed(0.0)))
            },
            chevron_element,
            icon_element,
            container(
                text(&row_data.display_name)
                    .size(ROW_NAME_SIZE)
                    .font(name_font)
                    .color(ctx.text_color)
                    .wrapping(iced::widget::text::Wrapping::None),
            )
            .width(Length::Fill)
            .clip(true),
        ]
        .spacing(ROW_NAME_SPACING)
        .align_y(Alignment::Center)
        .width(Length::Fill),
        container(
            text(&row_data.kind)
                .size(ROW_TEXT_SIZE)
                .font(ctx.main_font)
                .color(ctx.dim_color)
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .width(Length::Fixed(COLUMN_WIDTH_KIND))
        .clip(true),
        text(&row_data.size)
            .size(ROW_TEXT_SIZE)
            .font(ctx.main_font)
            .color(size_color)
            .width(Length::Fixed(COLUMN_WIDTH_SIZE))
            .align_x(alignment::Horizontal::Right),
        container(
            text(&row_data.modified)
                .size(ROW_TEXT_SIZE)
                .font(ctx.main_font)
                .color(ctx.sub_dim_color)
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .width(Length::Fixed(COLUMN_WIDTH_MODIFIED))
        .clip(true),
    ]
    .align_y(Alignment::Center)
    .height(Length::Fill)
    .padding(ROW_CONTENT_PADDING)
    .spacing(ROW_CONTENT_SPACING)
    .width(Length::Fill);

    button(row_content)
        .on_press(crate::app::messages::NavigationMsg::FileClicked(row_index).into())
        .style(move |theme, status| folder_row_style(theme, status, is_selected, is_odd))
        .padding(0)
        .height(ROW_BUTTON_HEIGHT)
        .width(Length::Fill)
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
        primitive::WHITE_008
    } else {
        primitive::BLACK_008
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

fn render_row_icon<'a>(icon_name: &str, is_dir: bool, is_expanded: bool) -> Element<'a, Message> {
    let resolved_icon = if is_dir {
        if is_expanded {
            icon_theme::get_icon_handle("folder-open")
                .or_else(|| icon_theme::get_icon_handle("inode-directory"))
        } else {
            icon_theme::get_icon_handle("folder")
                .or_else(|| icon_theme::get_icon_handle("inode-directory"))
        }
    } else {
        icon_theme::get_icon_handle(icon_name).or_else(|| {
            icon_theme::get_icon_handle("text-x-generic")
                .or_else(|| icon_theme::get_icon_handle("application-x-generic"))
        })
    };

    if let Some(svg_handle) = resolved_icon {
        svg(svg_handle)
            .width(ROW_ICON_SIZE)
            .height(ROW_ICON_SIZE)
            .into()
    } else {
        let fallback = if is_dir {
            if is_expanded { "📂" } else { "📁" }
        } else {
            "📄"
        };
        text(fallback).size(ROW_ICON_SIZE).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FolderRowState;

    #[test]
    fn test_view_folder_with_long_file_name_constructs_without_panic() {
        let long_name = "a".repeat(300) + ".tar.gz";
        let long_kind = "A".repeat(150);
        let long_modified = "2026-11-14 15:30:00.123456789 UTC+07:00 (Indochina Time)";

        let row_data = FolderRowState {
            name: long_name.clone(),
            display_name: long_name,
            kind: long_kind,
            size: "1.2 GB".to_string(),
            raw_size: 1_200_000_000,
            modified: long_modified.to_string(),
            raw_modified: 1731580000,
            path: "/tmp/long_file".to_string(),
            is_dir: false,
            icon: "application-x-tar",
            depth: 0,
            parent_path: None,
        };

        let state = FolderState {
            rows: vec![row_data],
            folder_path: "/tmp".to_string(),
            ..Default::default()
        };

        let _element = view_folder(&state, AppTheme::Dark, None);
    }

    #[test]
    fn test_view_folder_virtual_loading_windowing() {
        let rows: Vec<FolderRowState> = (0..500)
            .map(|i| FolderRowState {
                name: format!("file_{i}.txt"),
                display_name: format!("file_{i}.txt"),
                kind: "Text Document".to_string(),
                size: "10 KB".to_string(),
                raw_size: 10240,
                modified: "2026-11-14".to_string(),
                raw_modified: 1731580000,
                path: format!("/tmp/file_{i}.txt"),
                is_dir: false,
                icon: "text-x-generic",
                depth: 0,
                parent_path: None,
            })
            .collect();

        let state = FolderState {
            rows,
            folder_path: "/tmp".to_string(),
            scroll_y: 1900.0, // Scrolled down into the list
            viewport_height: 800.0,
            ..Default::default()
        };

        let _element = view_folder(&state, AppTheme::Dark, None);
    }
}
