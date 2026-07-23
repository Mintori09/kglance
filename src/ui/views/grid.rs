use std::path::{Path, PathBuf};
use iced::widget::{column, row, scrollable, text, container, button, image, responsive};
use iced::{Element, Length, Alignment};
use crate::app::Message;
use crate::core::types::GridThumbnail;

pub fn get_freedesktop_thumbnail_path(file_path: &str) -> Option<PathBuf> {
    let canonical = Path::new(file_path).canonicalize().ok()?;
    let uri = format!("file://{}", canonical.display());
    let digest = md5::compute(uri.as_bytes());
    let hex_hash = format!("{:x}.png", digest);
    
    let home = std::env::var("HOME").ok()?;
    let large = PathBuf::from(&home).join(".cache/thumbnails/large").join(&hex_hash);
    if large.exists() {
        return Some(large);
    }
    let normal = PathBuf::from(&home).join(".cache/thumbnails/normal").join(&hex_hash);
    if normal.exists() {
        return Some(normal);
    }
    None
}

pub fn view_grid<'a>(thumbnails: &'a [GridThumbnail], _active_index: usize) -> Element<'a, Message> {
    responsive(move |bounds| {
        let item_width = 150.0;
        let gap = 12.0;
        let cols = ((bounds.width - gap) / (item_width + gap)).floor().max(1.0) as usize;

        let mut col_widget = column![].spacing(gap).padding(gap);

        for (chunk_idx, chunk) in thumbnails.chunks(cols).enumerate() {
            let mut row_widget = row![].spacing(gap);
            for (item_idx, item) in chunk.iter().enumerate() {
                let global_idx = chunk_idx * cols + item_idx;

                let card_content: Element<'_, Message> = if let Some(ref handle) = item.thumbnail_handle {
                    image(handle.clone())
                        .width(140.0)
                        .height(100.0)
                        .content_fit(iced::ContentFit::Contain)
                        .into()
                } else {
                    container(text(&item.name).size(12))
                        .width(140.0)
                        .height(100.0)
                        .center_x(140.0)
                        .center_y(100.0)
                        .into()
                };


                let card_btn = button(
                    column![
                        card_content,
                        text(&item.name).size(11).line_height(1.2),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(4)
                )
                .on_press(Message::FileClickedInGrid(global_idx))
                .width(item_width);

                row_widget = row_widget.push(card_btn);
            }
            col_widget = col_widget.push(row_widget);
        }

        scrollable(col_widget).height(Length::Fill).width(Length::Fill).into()
    })
    .into()
}
