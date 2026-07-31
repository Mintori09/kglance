use crate::app::Message;
use crate::core::PdfState;
use crate::ui::components::scroll_pane::scroll_pane;
use iced::widget::{column, container, image, text};
use iced::{Element, Length};

use crate::ui::theme::tokens::{spacing, typography};

const PAGE_SPACING: f32 = spacing::S;
const PAGE_CONTAINER_PADDING: f32 = spacing::XS;
const PLACEHOLDER_PADDING: f32 = spacing::S;
const MAIN_COLUMN_PADDING: f32 = spacing::M;

const PLACEHOLDER_HEIGHT: f32 = 200.0;

const EMPTY_STATE_TEXT_SIZE: f32 = typography::SIZE_BODY;
const LOADING_TEXT_SIZE: f32 = typography::SIZE_BODY;
const PLACEHOLDER_TEXT_SIZE: f32 = typography::SIZE_SMALL;

const SCROLL_PANE_ID: &str = "content_scroll";
const EMPTY_STATE_MESSAGE: &str = "No pages";
const LOADING_MESSAGE: &str = "Loading…";

pub fn view_pdf<'a>(state: &'a PdfState) -> Element<'a, Message> {
    if state.page_count == 0 {
        return render_empty_state();
    }

    let mut pages_column = column![].spacing(PAGE_SPACING).padding(MAIN_COLUMN_PADDING);

    for (page_index, page_entry) in state.pages.iter().take(state.page_count).enumerate() {
        let page_card = match page_entry {
            Some(entry) => render_page_image(&entry.handle),
            None => render_page_placeholder(page_index + 1),
        };
        pages_column = pages_column.push(page_card);
    }

    if state.loading {
        pages_column = pages_column.push(render_loading_indicator());
    }

    scroll_pane(
        SCROLL_PANE_ID,
        container(pages_column)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_scroll(Message::PdfScrolled)
    .build()
}

fn render_empty_state<'a>() -> Element<'a, Message> {
    scroll_pane(
        SCROLL_PANE_ID,
        text(EMPTY_STATE_MESSAGE).size(EMPTY_STATE_TEXT_SIZE),
    )
    .build()
}

fn render_page_image<'a>(image_handle: &image::Handle) -> Element<'a, Message> {
    let page_image = image(image_handle.clone())
        .width(Length::Shrink)
        .height(Length::Shrink);

    container(page_image)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding(PAGE_CONTAINER_PADDING)
        .into()
}

fn render_page_placeholder<'a>(page_number: usize) -> Element<'a, Message> {
    let placeholder_text = text(format!("Page {}…", page_number))
        .size(PLACEHOLDER_TEXT_SIZE)
        .center();

    container(placeholder_text)
        .width(Length::Fill)
        .height(Length::Fixed(PLACEHOLDER_HEIGHT))
        .padding(PLACEHOLDER_PADDING)
        .into()
}

fn render_loading_indicator<'a>() -> Element<'a, Message> {
    text(LOADING_MESSAGE)
        .size(LOADING_TEXT_SIZE)
        .width(Length::Fill)
        .center()
        .into()
}
