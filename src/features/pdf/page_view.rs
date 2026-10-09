use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse::{self, click};
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget, tree};
use iced::advanced::graphics::core::{Clipboard, Element, Shell};
use iced::advanced::image;
use iced::{Background, Border, Color, Length, Rectangle, Shadow, Size};

use crate::features::pdf::selection::{
    PdfPageText, PdfPosition, PdfSelection, compute_selection_rects,
};
use crate::features::pdf::types::PageDimensions;

#[derive(Default)]
struct PageWidgetState {
    is_mouse_held: bool,
    last_click: Option<mouse::Click>,
}

pub struct PdfPageWidget<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: renderer::Renderer + image::Renderer<Handle = iced::widget::image::Handle>,
{
    page_index: usize,
    image_handle: Option<&'a iced::widget::image::Handle>,
    page_text: Option<&'a PdfPageText>,
    page_dimensions: PageDimensions,
    width: f32,
    height: f32,
    selection: Option<PdfSelection>,
    selection_color: Color,
    is_selecting: bool,
    on_drag_start: Option<Box<dyn Fn(PdfPosition) -> Message + 'a>>,
    on_drag_update: Option<Box<dyn Fn(PdfPosition) -> Message + 'a>>,
    on_drag_end: Option<Box<dyn Fn() -> Message + 'a>>,
    on_clear_selection: Option<Box<dyn Fn() -> Message + 'a>>,
    _phantom: std::marker::PhantomData<(Theme, Renderer)>,
}

impl<'a, Message, Theme, Renderer> PdfPageWidget<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + image::Renderer<Handle = iced::widget::image::Handle>,
{
    pub fn new(
        page_index: usize,
        image_handle: Option<&'a iced::widget::image::Handle>,
        page_text: Option<&'a PdfPageText>,
        page_dimensions: PageDimensions,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            page_index,
            image_handle,
            page_text,
            page_dimensions,
            width,
            height,
            selection: None,
            selection_color: crate::ui::theme::color::primitive::SELECTION_DARK_BG,
            is_selecting: false,
            on_drag_start: None,
            on_drag_update: None,
            on_drag_end: None,
            on_clear_selection: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn selection(mut self, selection: Option<PdfSelection>) -> Self {
        self.selection = selection;
        self
    }

    pub fn is_selecting(mut self, is_selecting: bool) -> Self {
        self.is_selecting = is_selecting;
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    pub fn on_drag_start(mut self, on_drag_start: impl Fn(PdfPosition) -> Message + 'a) -> Self {
        self.on_drag_start = Some(Box::new(on_drag_start));
        self
    }

    pub fn on_drag_update(mut self, on_drag_update: impl Fn(PdfPosition) -> Message + 'a) -> Self {
        self.on_drag_update = Some(Box::new(on_drag_update));
        self
    }

    pub fn on_drag_end(mut self, on_drag_end: impl Fn() -> Message + 'a) -> Self {
        self.on_drag_end = Some(Box::new(on_drag_end));
        self
    }

    pub fn on_clear_selection(mut self, on_clear_selection: impl Fn() -> Message + 'a) -> Self {
        self.on_clear_selection = Some(Box::new(on_clear_selection));
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PdfPageWidget<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + image::Renderer<Handle = iced::widget::image::Handle>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PageWidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PageWidgetState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.width, self.height))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<PageWidgetState>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    let rel_x = cursor_pos.x - bounds.x;
                    let rel_y = cursor_pos.y - bounds.y;
                    let scale = (self.width / self.page_dimensions.width_pts.max(1.0)).max(0.001);
                    let pt_x = rel_x / scale;
                    let pt_y = rel_y / scale;

                    if let Some(page_text) = self.page_text
                        && let Some((line_idx, char_idx)) = page_text.hit_test(pt_x, pt_y)
                    {
                        let pos = PdfPosition {
                            page: self.page_index,
                            line: line_idx,
                            char_idx,
                        };
                        let new_click =
                            mouse::Click::new(cursor_pos, mouse::Button::Left, state.last_click);

                        match new_click.kind() {
                            click::Kind::Double => {
                                let (w_start, w_end) =
                                    page_text.find_word_range(line_idx, char_idx);
                                let start_pos = PdfPosition {
                                    page: self.page_index,
                                    line: line_idx,
                                    char_idx: w_start,
                                };
                                let end_pos = PdfPosition {
                                    page: self.page_index,
                                    line: line_idx,
                                    char_idx: w_end,
                                };
                                state.is_mouse_held = false;

                                if let Some(ref on_start) = self.on_drag_start {
                                    shell.publish(on_start(start_pos));
                                }
                                if let Some(ref on_update) = self.on_drag_update {
                                    shell.publish(on_update(end_pos));
                                }
                                if let Some(ref on_end) = self.on_drag_end {
                                    shell.publish(on_end());
                                }
                            }
                            click::Kind::Triple => {
                                let line_char_count = page_text
                                    .lines
                                    .get(line_idx)
                                    .map(|l| l.chars.len())
                                    .unwrap_or(0);
                                let start_pos = PdfPosition {
                                    page: self.page_index,
                                    line: line_idx,
                                    char_idx: 0,
                                };
                                let end_pos = PdfPosition {
                                    page: self.page_index,
                                    line: line_idx,
                                    char_idx: line_char_count,
                                };
                                state.is_mouse_held = false;

                                if let Some(ref on_start) = self.on_drag_start {
                                    shell.publish(on_start(start_pos));
                                }
                                if let Some(ref on_update) = self.on_drag_update {
                                    shell.publish(on_update(end_pos));
                                }
                                if let Some(ref on_end) = self.on_drag_end {
                                    shell.publish(on_end());
                                }
                            }
                            click::Kind::Single => {
                                state.is_mouse_held = true;

                                if let Some(ref on_start) = self.on_drag_start {
                                    shell.publish(on_start(pos));
                                }
                            }
                        }

                        state.last_click = Some(new_click);
                        shell.capture_event();
                    } else {
                        state.is_mouse_held = false;
                        if let Some(ref on_clear) = self.on_clear_selection {
                            shell.publish(on_clear());
                        }
                    }
                } else {
                    state.is_mouse_held = false;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !self.is_selecting {
                    state.is_mouse_held = false;
                }
                if (state.is_mouse_held || self.is_selecting)
                    && let Some(cursor_pos) = cursor.position()
                {
                    let is_over_this_page = cursor.position_over(bounds).is_some();
                    if is_over_this_page {
                        let rel_x = cursor_pos.x - bounds.x;
                        let rel_y = cursor_pos.y - bounds.y;
                        let scale =
                            (self.width / self.page_dimensions.width_pts.max(1.0)).max(0.001);
                        let pt_x = rel_x / scale;
                        let pt_y = rel_y / scale;

                        if let Some(page_text) = self.page_text
                            && let Some((line_idx, char_idx)) = page_text.hit_test(pt_x, pt_y)
                        {
                            let pos = PdfPosition {
                                page: self.page_index,
                                line: line_idx,
                                char_idx,
                            };
                            if let Some(ref on_update) = self.on_drag_update {
                                shell.publish(on_update(pos));
                            }
                            shell.capture_event();
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let was_held = state.is_mouse_held;
                state.is_mouse_held = false;

                if was_held && let Some(ref on_end) = self.on_drag_end {
                    shell.publish(on_end());
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // 1. Draw page image or placeholder
        if let Some(handle) = self.image_handle {
            renderer.draw_image(
                image::Image {
                    handle: handle.clone(),
                    filter_method: image::FilterMethod::Linear,
                    rotation: iced::Radians(0.0),
                    border_radius: crate::ui::theme::tokens::radius::SM.into(),
                    opacity: 1.0,
                    snap: false,
                },
                bounds,
                bounds,
            );
        } else {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: crate::ui::theme::tokens::radius::SM.into(),
                        width: 1.0,
                        color: Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                },
                Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.4)),
            );
        }

        // 2. Draw selection highlights (bôi đen)
        if let (Some(selection), Some(page_text)) = (self.selection, self.page_text)
            && !selection.is_collapsed()
        {
            let scale = (self.width / self.page_dimensions.width_pts.max(1.0)).max(0.001);
            let rects = compute_selection_rects(
                page_text,
                self.page_index,
                &selection,
                scale,
                bounds.x,
                bounds.y,
            );

            renderer.with_layer(bounds, |renderer| {
                for rect in rects {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: rect,
                            border: Border {
                                radius: crate::ui::theme::tokens::radius::XS.into(),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        Background::Color(self.selection_color),
                    );
                }
            });
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let widget_state = state.state.downcast_ref::<PageWidgetState>();
        let bounds = layout.bounds();

        if self.is_selecting || widget_state.is_mouse_held {
            mouse::Interaction::Text
        } else if let Some(cursor_pos) = cursor.position_over(bounds) {
            let rel_x = cursor_pos.x - bounds.x;
            let rel_y = cursor_pos.y - bounds.y;
            let scale = (self.width / self.page_dimensions.width_pts.max(1.0)).max(0.001);
            let pt_x = rel_x / scale;
            let pt_y = rel_y / scale;

            if self
                .page_text
                .is_some_and(|pt| pt.is_point_over_text(pt_x, pt_y))
            {
                mouse::Interaction::Text
            } else {
                mouse::Interaction::Idle
            }
        } else {
            mouse::Interaction::Idle
        }
    }
}

impl<'a, Message, Theme, Renderer> From<PdfPageWidget<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + image::Renderer<Handle = iced::widget::image::Handle> + 'a,
{
    fn from(widget: PdfPageWidget<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}
