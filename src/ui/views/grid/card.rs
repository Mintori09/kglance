use crate::app::Message;
use crate::core::types::GridThumbnail;
use crate::ui::theme::icon_theme::icon_for_entry;
use crate::ui::theme::{default_grid_card, icon_theme};
use crate::ui::views::grid::calculate::truncate_middle;
use crate::ui::views::grid::constants::*;
use iced::widget::{button, column, container, image, svg, text};
use iced::{Alignment, Element, Length, Theme};

pub fn create_grid_card<'a>(
    item: &'a GridThumbnail,
    item_index: usize,
    is_active: bool,
    dimensions: &ScaledDimensions,
) -> button::Button<'a, Message> {
    let card_media = render_card_media(item, dimensions);
    let card_label = text(truncate_middle(&item.name, MAX_LABEL_LENGTH))
        .size(dimensions.font_size)
        .shaping(text::Shaping::Advanced)
        .width(Length::Fill)
        .align_x(Alignment::Center);

    let card_content = column![card_media, card_label]
        .align_x(Alignment::Center)
        .spacing(CARD_INNER_SPACING)
        .padding(CARD_PADDING);

    button(card_content)
        .on_press(crate::app::messages::NavigationMsg::FileClickedInGrid(item_index).into())
        .style(move |theme: &Theme, status: button::Status| {
            default_grid_card(theme, status, is_active)
        })
        .width(dimensions.item_width)
}

pub fn render_card_media<'a>(
    item: &'a GridThumbnail,
    dimensions: &ScaledDimensions,
) -> Element<'a, Message> {
    let media_element: Element<'a, Message> = if let Some(ref handle) = item.thumbnail_handle {
        image(handle.clone())
            .width(dimensions.image_width)
            .height(dimensions.image_height)
            .content_fit(iced::ContentFit::Contain)
            .into()
    } else {
        render_fallback_icon(&item.name, dimensions.icon_size)
    };

    container(media_element)
        .width(dimensions.container_width)
        .height(dimensions.container_height)
        .center_x(dimensions.container_width)
        .center_y(dimensions.container_height)
        .into()
}

fn render_fallback_icon<'a>(item_name: &str, icon_size: f32) -> Element<'a, Message> {
    let icon_name = icon_for_entry(item_name, false);
    if let Some(svg_handle) = icon_theme::get_icon_handle(icon_name) {
        svg(svg_handle).width(icon_size).height(icon_size).into()
    } else {
        text(DEFAULT_FILE_ICON_EMOJI).size(icon_size * 0.75).into()
    }
}
