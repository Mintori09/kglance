use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme::{default_button, default_text_input};

const DEFAULT_SPACING: f32 = 10.0;
const DEFAULT_PADDING: u16 = 5;
const INFO_TEXT_SIZE: f32 = 14.0;

const LABEL_NEXT: &str = "Next";
const LABEL_PREV: &str = "Prev";
const LABEL_CLOSE: &str = "Close";

pub enum SearchKind {
    Text,
    Markdown,
    Json,
    Spreadsheet,
    Grid,
}

struct SearchMessages {
    on_query: fn(String) -> Message,
    on_next: Option<Message>,
    on_prev: Option<Message>,
    on_close: Message,
}

impl SearchKind {
    fn placeholder(&self) -> &'static str {
        match self {
            Self::Text => "Search...",
            Self::Markdown => "Search markdown...",
            Self::Json => "Search key or value...",
            Self::Spreadsheet => "Search spreadsheet...",
            Self::Grid => "Search files...",
        }
    }

    fn input_id(&self) -> &'static str {
        match self {
            Self::Text => "txt_search_input",
            Self::Markdown => "md_search_input",
            Self::Json => "json_search_input",
            Self::Spreadsheet => "ss_search_input",
            Self::Grid => "grid_search_input",
        }
    }

    fn messages(&self) -> SearchMessages {
        match self {
            Self::Text => SearchMessages {
                on_query: Message::SearchQueryChanged,
                on_next: Some(Message::TextSearchNext),
                on_prev: Some(Message::TextSearchPrev),
                on_close: Message::TextSearchClosed,
            },
            Self::Markdown => SearchMessages {
                on_query: Message::MarkdownSearchQueryChanged,
                on_next: Some(Message::MarkdownSearchNext),
                on_prev: Some(Message::MarkdownSearchPrev),
                on_close: Message::MarkdownSearchClosed,
            },
            Self::Json => SearchMessages {
                on_query: Message::JsonSearchQueryChanged,
                on_next: None,
                on_prev: None,
                on_close: Message::JsonSearchClosed,
            },
            Self::Spreadsheet => SearchMessages {
                on_query: Message::SpreadsheetSearchQueryChanged,
                on_next: None,
                on_prev: None,
                on_close: Message::SpreadsheetSearchClosed,
            },
            Self::Grid => SearchMessages {
                on_query: Message::GridSearchQueryChanged,
                on_next: None,
                on_prev: None,
                on_close: Message::GridSearchClosed,
            },
        }
    }
}

pub fn search_bar<'a>(
    kind: SearchKind,
    query: &'a str,
    info: Option<&'a str>,
) -> Element<'a, Message> {
    let messages = kind.messages();
    let input_id = kind.input_id();

    let query_input: Element<'a, Message> = text_input(kind.placeholder(), query)
        .id(input_id)
        .on_input(messages.on_query)
        .style(default_text_input)
        .width(Length::Fill)
        .into();

    let mut elements: Vec<Element<'a, Message>> = vec![query_input];

    if let Some(next_msg) = messages.on_next {
        elements.push(build_action_button(LABEL_NEXT, next_msg));
    }

    if let Some(prev_msg) = messages.on_prev {
        elements.push(build_action_button(LABEL_PREV, prev_msg));
    }

    if let Some(info_text) = info {
        elements.push(text(info_text).size(INFO_TEXT_SIZE).into());
    }

    elements.push(build_action_button(LABEL_CLOSE, messages.on_close));

    container(
        row(elements)
            .spacing(DEFAULT_SPACING)
            .padding(DEFAULT_PADDING),
    )
    .width(Length::Fill)
    .into()
}

fn build_action_button<'a>(label: &'static str, on_press: Message) -> Element<'a, Message> {
    button(text(label))
        .on_press(on_press)
        .style(default_button)
        .into()
}
