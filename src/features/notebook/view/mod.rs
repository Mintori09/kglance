pub mod cell;
pub mod output;

use crate::app::Message;
use crate::features::notebook::types::NotebookData;
use crate::features::notebook::view::cell::render_cell;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::theme::AppTheme;
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding};

pub const NOTEBOOK_SCROLL_ID: &str = "content_scroll";

pub fn view_notebook<'a>(
    data: &'a NotebookData,
    md_state: &'a crate::core::MarkdownState,
    font_size: f32,
    theme: AppTheme,
    font_family: Option<&'a str>,
    font_family_mono: Option<&'a str>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    if data.cells.is_empty() {
        let base = theme.palette().base;
        return container(
            text("Notebook is empty (0 cells)")
                .size(font_size)
                .color(base.text_dim),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let mut cells_col = column![].spacing(16);

    for cell in &data.cells {
        cells_col = cells_col.push(render_cell(
            cell,
            md_state,
            font_size,
            theme,
            font_family,
            font_family_mono,
        ));
    }

    let content_container = if let Some(max_w) = max_text_width {
        container(cells_col)
            .max_width(max_w)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
    } else {
        container(cells_col).width(Length::Fill)
    };

    let padded = container(content_container)
        .padding(Padding::from([16, 24]))
        .width(Length::Fill);

    scroll_pane(NOTEBOOK_SCROLL_ID, padded).build()
}
