use crate::app::Message;
use crate::core::PdfState;
use crate::core::types::PdfSidebarMode;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{drag_handle, sidebar_entry_style};
use crate::ui::theme::tokens::spacing;
use iced::widget::{button, column, container, image, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

const PAGE_SPACING: f32 = spacing::S;
const MAIN_COLUMN_PADDING: f32 = spacing::M;

const EMPTY_STATE_TEXT_SIZE: f32 = 14.0;
const PLACEHOLDER_TEXT_SIZE: f32 = 12.0;

const SCROLL_PANE_ID: &str = "content_scroll";
const EMPTY_STATE_MESSAGE: &str = "No pages";

/// Recalculate page Y offsets when display width changes (e.g. zoom/font_size change).
pub fn recalculate_pdf_offsets(pdf_state: &mut crate::core::PdfState, font_size: f32) {
    if pdf_state.page_dimensions.is_empty() {
        return;
    }
    let display_width = (font_size / 14.0) * 800.0;
    let (offsets, ends, total_h) = crate::core::preview::compute_pdf_page_offsets(
        &pdf_state.page_dimensions,
        display_width,
        PAGE_SPACING,
    );
    pdf_state.display_width = display_width;
    pdf_state.page_y_offsets = offsets;
    pdf_state.page_ends = ends;
    pdf_state.total_content_height = total_h;
}

/// Recalculate PDF layout for a new font size and return the adjusted scroll Y offset to anchor the visible position.
pub fn rescale_pdf_and_anchor(
    pdf_state: &mut crate::core::PdfState,
    old_font_size: f32,
    new_font_size: f32,
) -> f32 {
    if pdf_state.page_dimensions.is_empty() || pdf_state.page_count == 0 {
        return 0.0;
    }

    let current_scroll_y = pdf_state.scroll_y;
    let current_page = crate::features::pdf::viewport::find_visible_page(
        &pdf_state.page_y_offsets,
        current_scroll_y,
        800.0,
        0.0,
    );

    let old_page_y = pdf_state
        .page_y_offsets
        .get(current_page)
        .copied()
        .unwrap_or(0.0);
    let old_display_w = (old_font_size / 14.0) * 800.0;
    let old_page_h = pdf_state
        .page_dimensions
        .get(current_page)
        .map(|d| d.display_height(old_display_w))
        .unwrap_or(800.0);

    let fractional_progress = if old_page_h > 0.0 {
        ((current_scroll_y - old_page_y) / old_page_h).clamp(0.0, 1.0)
    } else {
        0.0
    };

    recalculate_pdf_offsets(pdf_state, new_font_size);

    let new_page_y = pdf_state
        .page_y_offsets
        .get(current_page)
        .copied()
        .unwrap_or(0.0);
    let new_display_w = (new_font_size / 14.0) * 800.0;
    let new_page_h = pdf_state
        .page_dimensions
        .get(current_page)
        .map(|d| d.display_height(new_display_w))
        .unwrap_or(800.0);

    let new_scroll_y = (new_page_y + fractional_progress * new_page_h).max(0.0);
    pdf_state.scroll_y = new_scroll_y;
    new_scroll_y
}

pub fn view_pdf<'a>(
    state: &'a PdfState,
    font_size: f32,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    let pages_view = view_pdf_pages(state, SCROLL_PANE_ID, font_size, |vp| {
        crate::app::messages::PdfMsg::Scrolled(vp).into()
    });

    if state.sidebar_visible {
        let sidebar = render_pdf_sidebar(state, theme);
        let d_handle = drag_handle(state.sidebar_resizing, theme, Message::SidebarDragStarted);
        row![sidebar, d_handle, pages_view]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        pages_view
    }
}

pub fn view_pdf_pages<'a>(
    state: &'a PdfState,
    scroll_id: &'static str,
    font_size: f32,
    on_scroll: impl Fn(iced::widget::scrollable::Viewport) -> Message + 'static,
) -> Element<'a, Message> {
    if state.page_count == 0 {
        return render_empty_state(scroll_id);
    }

    let page_width = if state.display_width > 0.0 {
        state.display_width
    } else {
        (font_size / 14.0) * 800.0
    };
    let view_h = if state.viewport_height > 0.0 {
        state.viewport_height
    } else {
        800.0
    };

    let visible_opt = crate::features::pdf::geometry::visible_page_range(
        &state.page_y_offsets,
        &state.page_ends,
        state.scroll_y,
        view_h,
    );

    let render_range = crate::features::pdf::geometry::buffered_page_range(
        visible_opt,
        state.page_count,
        2, // BUFFER_PAGES
    )
    .unwrap_or(0..=0);

    let layout = crate::features::pdf::geometry::calculate_virtualized_layout(
        &state.page_y_offsets,
        state.total_content_height,
        PAGE_SPACING,
        render_range,
    );

    let mut pages_column = column![].spacing(PAGE_SPACING).padding(0.0);

    // Top spacer
    if layout.top_spacer_height > 0.0 {
        pages_column = pages_column.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(layout.top_spacer_height),
        );
    }

    // Render visible pages
    for page_index in layout.first_render..=layout.last_render {
        if page_index >= state.page_count {
            break;
        }
        let aspect_ratio = state
            .page_dimensions
            .get(page_index)
            .map(|d| d.aspect_ratio())
            .unwrap_or(1.0 / 1.414);
        let page_height = page_width / aspect_ratio;

        let page_card = match state.pages.get(page_index) {
            Some(entry) => render_page_image(&entry.handle, page_width, page_height),
            None => render_page_placeholder(page_index + 1, page_width, page_height),
        };
        pages_column = pages_column.push(page_card);
    }

    // Bottom spacer
    if layout.bottom_spacer_height > 0.0 {
        pages_column = pages_column.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(layout.bottom_spacer_height),
        );
    }

    let centered = container(pages_column)
        .center_x(Length::Fill)
        .width(Length::Fill);

    let content_scroll = scroll_pane(scroll_id, centered)
        .on_scroll(on_scroll)
        .build();

    container(content_scroll)
        .padding(MAIN_COLUMN_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_pdf_sidebar<'a>(
    state: &'a PdfState,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    let tabs = render_sidebar_tabs(state.sidebar_mode, theme);

    let content: Element<'a, Message> = match state.sidebar_mode {
        PdfSidebarMode::Thumbnails => render_thumbnails_list(state, theme),
        PdfSidebarMode::Toc => render_toc_list(state, theme),
    };

    let p = theme.palette().base;
    let sidebar_container = column![tabs, content]
        .width(state.sidebar_width)
        .height(Length::Fill);

    iced::widget::opaque(
        container(sidebar_container)
            .width(state.sidebar_width)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(p.bg.into()),
                border: Border {
                    width: 1.0,
                    color: p.border,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
    )
}

fn render_sidebar_tabs<'a>(
    active_mode: PdfSidebarMode,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    let thumbs_btn = button(
        container(
            row![text("🖼").size(12), text("Thumbs").size(11)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .center_x(Length::Fill),
    )
    .on_press(crate::app::messages::PdfMsg::SetSidebarMode(PdfSidebarMode::Thumbnails).into())
    .width(Length::Fill)
    .style(move |iced_theme, status| {
        sidebar_entry_style(
            iced_theme,
            status,
            active_mode == PdfSidebarMode::Thumbnails,
            theme,
        )
    })
    .padding([6, 10]);

    let toc_btn = button(
        container(
            row![text("≡").size(12), text("TOC").size(11)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .center_x(Length::Fill),
    )
    .on_press(crate::app::messages::PdfMsg::SetSidebarMode(PdfSidebarMode::Toc).into())
    .width(Length::Fill)
    .style(move |iced_theme, status| {
        sidebar_entry_style(
            iced_theme,
            status,
            active_mode == PdfSidebarMode::Toc,
            theme,
        )
    })
    .padding([6, 10]);

    container(
        row![thumbs_btn, toc_btn]
            .spacing(4)
            .padding(6)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}

fn render_thumbnails_list<'a>(
    state: &'a PdfState,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    let current_page = state
        .visible_page
        .load(std::sync::atomic::Ordering::Relaxed);

    let mut thumbs_col = column![].spacing(10).padding(8);

    for page_idx in 0..state.page_count {
        let is_active = page_idx == current_page;
        let thumb_item = render_thumb_item(state, page_idx, is_active, theme);
        thumbs_col = thumbs_col.push(thumb_item);
    }

    scroll_pane("pdf_thumb_scroll", thumbs_col).build()
}

fn render_thumb_item<'a>(
    state: &'a PdfState,
    page_idx: usize,
    is_active: bool,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    let thumb_width = (state.sidebar_width - 24.0).clamp(100.0, 360.0);

    let thumb_img: Element<'a, Message> = if let Some(entry) = state.thumbnails.get(page_idx) {
        image(&entry.handle)
            .width(Length::Fixed(thumb_width))
            .height(Length::Shrink)
            .into()
    } else if let Some(entry) = state.pages.get(page_idx) {
        image(&entry.handle)
            .width(Length::Fixed(thumb_width))
            .height(Length::Shrink)
            .into()
    } else {
        container(text(format!("Page {}", page_idx + 1)).size(11))
            .width(Length::Fixed(thumb_width))
            .height(Length::Fixed(thumb_width * 1.3))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    let badge_bg = theme.palette().base.surface;
    let badge_text_color = theme.palette().base.text;

    let page_badge = container(text(format!("{}", page_idx + 1)).size(10).style(move |_| {
        iced::widget::text::Style {
            color: Some(badge_text_color),
        }
    }))
    .padding([2, 6])
    .style(move |_| container::Style {
        background: Some(badge_bg.into()),
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        ..Default::default()
    });

    let card_stack = iced::widget::stack![
        thumb_img,
        container(page_badge)
            .padding(4)
            .align_y(Alignment::End)
            .align_x(iced::alignment::Horizontal::Right)
    ];

    let border_color = if is_active {
        theme.palette().sidebar.active_text
    } else {
        iced::Color::TRANSPARENT
    };
    let border_width = if is_active { 2.0 } else { 0.0 };

    button(card_stack)
        .on_press(crate::app::messages::PdfMsg::ThumbnailClicked(page_idx).into())
        .width(Length::Fill)
        .style(move |iced_theme, status| {
            let mut style = sidebar_entry_style(iced_theme, status, is_active, theme);
            style.border = Border {
                radius: 6.0.into(),
                width: border_width,
                color: border_color,
            };
            style
        })
        .padding(2)
        .into()
}

fn render_toc_list<'a>(
    state: &'a PdfState,
    theme: crate::ui::theme::AppTheme,
) -> Element<'a, Message> {
    if state.outline.is_empty() {
        return container(text("No TOC available").size(12))
            .padding(12)
            .center_x(Length::Fill)
            .into();
    }

    let current_page = state
        .visible_page
        .load(std::sync::atomic::Ordering::Relaxed);

    let active_idx = state
        .outline
        .iter()
        .rposition(|entry| entry.page <= current_page);

    let mut toc_col = column![].spacing(2).padding(6);

    for (idx, entry) in state.outline.iter().enumerate() {
        let is_active = active_idx == Some(idx);
        let indent = (entry.level.saturating_sub(1) as f32) * 12.0;

        let label = text(&entry.title).size(12);
        let btn = button(label)
            .on_press(crate::app::messages::PdfMsg::TocItemClicked(entry.page).into())
            .width(Length::Fill)
            .style(move |iced_theme, status| {
                sidebar_entry_style(iced_theme, status, is_active, theme)
            })
            .padding([4, 6]);

        let row_item = container(btn)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: indent,
            })
            .width(Length::Fill);

        toc_col = toc_col.push(row_item);
    }

    scroll_pane("pdf_toc_scroll", toc_col).build()
}

fn render_empty_state<'a>(scroll_id: &'static str) -> Element<'a, Message> {
    scroll_pane(
        scroll_id,
        text(EMPTY_STATE_MESSAGE).size(EMPTY_STATE_TEXT_SIZE),
    )
    .build()
}

fn render_page_image<'a>(
    image_handle: &image::Handle,
    target_width: f32,
    page_height: f32,
) -> Element<'a, Message> {
    let page_image = image(image_handle.clone())
        .width(Length::Fixed(target_width))
        .height(Length::Fixed(page_height));

    let card = container(page_image)
        .width(Length::Fixed(target_width))
        .height(Length::Fixed(page_height))
        .center_x(Length::Fixed(target_width))
        .center_y(Length::Fixed(page_height));

    container(card)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

fn render_page_placeholder<'a>(
    page_number: usize,
    target_width: f32,
    page_height: f32,
) -> Element<'a, Message> {
    let placeholder_text = text(format!("Page {}…", page_number))
        .size(PLACEHOLDER_TEXT_SIZE)
        .center();

    let card = container(placeholder_text)
        .width(Length::Fixed(target_width))
        .height(Length::Fixed(page_height))
        .center_x(Length::Fixed(target_width))
        .center_y(Length::Fixed(page_height));

    container(card)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
