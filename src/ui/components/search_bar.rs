use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme::{breeze_button, breeze_text_input};

pub enum SearchKind {
    Text,
    Markdown,
    Json,
}

fn search_messages(kind: SearchKind) -> SearchMessages {
    match kind {
        SearchKind::Text => SearchMessages {
            on_query: Message::SearchQueryChanged,
            on_next: Some(Message::TextSearchNext),
            on_prev: Some(Message::TextSearchPrev),
            on_close: Message::TextSearchClosed,
        },
        SearchKind::Markdown => SearchMessages {
            on_query: Message::MarkdownSearchQueryChanged,
            on_next: Some(Message::MarkdownSearchNext),
            on_prev: Some(Message::MarkdownSearchPrev),
            on_close: Message::MarkdownSearchClosed,
        },
        SearchKind::Json => SearchMessages {
            on_query: Message::JsonSearchQueryChanged,
            on_next: None,
            on_prev: None,
            on_close: Message::JsonSearchClosed,
        },
    }
}

struct SearchMessages {
    on_query: fn(String) -> Message,
    on_next: Option<Message>,
    on_prev: Option<Message>,
    on_close: Message,
}

pub fn search_bar<'a>(
    kind: SearchKind,
    query: &'a str,
    info: Option<&'a str>,
    placeholder: &str,
) -> Element<'a, Message> {
    let msgs = search_messages(kind);

    let input: Element<'a, Message> = text_input(placeholder, query)
        .on_input(msgs.on_query)
        .style(breeze_text_input)
        .width(Length::Fill)
        .into();

    let mut items: Vec<Element<'a, Message>> = vec![input];

    if let Some(next) = msgs.on_next {
        items.push(
            button(text("Next"))
                .on_press(next)
                .style(breeze_button)
                .into(),
        );
    }
    if let Some(prev) = msgs.on_prev {
        items.push(
            button(text("Prev"))
                .on_press(prev)
                .style(breeze_button)
                .into(),
        );
    }
    if let Some(info_text) = info {
        items.push(text(info_text).size(14).into());
    }
    items.push(
        button(text("Close"))
            .on_press(msgs.on_close)
            .style(breeze_button)
            .into(),
    );

    container(row(items).spacing(10).padding(5))
        .width(Length::Fill)
        .into()
}
