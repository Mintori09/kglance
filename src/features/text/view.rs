pub mod outline;

use crate::app::Message;
use crate::core::TextState;
use crate::features::text::view::outline::render_outline_sidebar;
use crate::ui::components::code_viewer::VirtualCodeViewer;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, goto_line_bar, search_bar};
use crate::ui::theme::font::get_code_font;
use iced::Element;
use iced::widget::{column, row};

use crate::ui::theme::tokens::spacing;

const MAIN_CONTENT_SPACING: f32 = spacing::XS;
const SCROLL_PANE_PADDING: f32 = 0.0;

const SCROLL_PANE_ID: &str = "content_scroll";

pub fn view_text<'a>(
    state: &'a TextState,
    theme: crate::ui::theme::AppTheme,
    font_size: f32,
    font_family: Option<&str>,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    let font = get_code_font(font_family_mono);
    let mut main_content = column![]
        .spacing(MAIN_CONTENT_SPACING)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

    if state.search_visible {
        main_content = main_content.push(search_bar(
            SearchKind::Text,
            &state.search_query,
            Some(&state.search_info),
        ));
    }

    if state.goto_line_visible {
        main_content = main_content.push(goto_line_bar(
            &state.goto_line_query,
            state.document.total_lines().max(1),
        ));
    }

    let code_viewer = VirtualCodeViewer::<Message>::new(&state.document, font_size, font, theme)
        .display_map(&state.display_map)
        .wrap(word_wrap)
        .tokens(&state.cached_tokens, state.cached_tokens_start_line)
        .selection(state.selection)
        .search(
            &state.search_query,
            &state.search_matches,
            state.search_match_index,
        )
        .on_select(|sel| crate::app::messages::TextMsg::SelectionChanged(sel).into())
        .on_drag_start(|pos| crate::app::messages::TextMsg::SelectionDragStarted(pos).into())
        .on_drag_end(|| crate::app::messages::TextMsg::SelectionDragEnded.into())
        .on_auto_scroll(|delta, cursor| {
            crate::app::messages::TextMsg::AutoScroll(delta, cursor).into()
        })
        .on_copy(|text| crate::app::messages::ActionMsg::CopyText(text).into());

    let code_element: Element<'a, Message> = code_viewer.into();

    let scrollable_viewer = scroll_pane(SCROLL_PANE_ID, code_element)
        .filter_wheel(true)
        .container_padding(SCROLL_PANE_PADDING)
        .on_scroll(|viewport| crate::app::messages::TextMsg::Scrolled(viewport).into())
        .on_wheel(|delta| crate::app::messages::TextMsg::WheelScrolled(delta).into())
        .build();

    main_content = main_content.push(scrollable_viewer);

    if state.outline_visible {
        let sidebar = render_outline_sidebar(
            &state.symbols,
            theme,
            state.sidebar_width,
            state.scroll_y,
            &state.display_map,
            font_family,
        );
        let drag = crate::ui::components::sidebar::drag_handle(
            state.sidebar_resizing,
            theme,
            Message::SidebarDragStarted,
        );
        row![sidebar, drag, main_content]
            .spacing(0)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    } else {
        main_content.into()
    }
}
