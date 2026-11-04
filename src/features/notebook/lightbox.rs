use iced::Task;

use crate::app::{KglanceApp, Message};
use crate::features::lightbox::types::LightboxItem;
use crate::features::notebook::types::{CellOutput, NotebookCell, NotebookData};

pub fn collect_notebook_lightbox_items(
    data: &NotebookData,
    cached_handles: &std::collections::HashMap<String, iced::widget::image::Handle>,
) -> Vec<LightboxItem> {
    let mut items = Vec::new();

    for (cell_idx, cell) in data.cells.iter().enumerate() {
        match cell {
            NotebookCell::Markdown { blocks, .. } => {
                let md_items = crate::features::markdown::lightbox::collect_markdown_lightbox_items(
                    blocks,
                    cached_handles,
                );
                items.extend(md_items);
            }
            NotebookCell::Code {
                execution_count,
                outputs,
                ..
            } => {
                for (out_idx, output) in outputs.iter().enumerate() {
                    if let CellOutput::Image {
                        data, format_hint, ..
                    } = output
                    {
                        let id = format!("cell_{cell_idx}_out_{out_idx}");
                        let title = execution_count.map(|ec| format!("Out [{ec}]"));
                        let subtitle = format!("Cell {} Output ({format_hint})", cell_idx + 1);
                        items.push(LightboxItem {
                            id,
                            title,
                            subtitle,
                            link_url: None,
                            in_memory_bytes: Some(data.clone()),
                        });
                    }
                }
            }
            NotebookCell::Raw { .. } => {}
        }
    }

    items
}

pub fn handle_open_notebook_image_lightbox(app: &mut KglanceApp, target_id: &str) -> Task<Message> {
    let items = {
        let Some(crate::core::PreviewData::Notebook(ref data)) = app.current_content else {
            return Task::none();
        };
        collect_notebook_lightbox_items(data, &app.state.markdown.cached_image_handles)
    };

    if items.is_empty() {
        return Task::none();
    }

    let initial_index = items.iter().position(|it| it.id == target_id).unwrap_or(0);
    crate::features::lightbox::ops::handle_open_lightbox(app, items, initial_index)
}

#[cfg(test)]
#[path = "lightbox_tests.rs"]
mod tests;
