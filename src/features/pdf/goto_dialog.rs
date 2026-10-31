use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

use crate::app::Message;
use crate::app::messages::PdfMsg;
use crate::ui::theme::tokens::{radius, spacing, typography};
use crate::ui::theme::{default_button, default_text_input};

pub const GOTO_PAGE_INPUT_ID: &str = "pdf_goto_page_input";

/// State for the Go to Page dialog popover.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GotoPageState {
    pub is_visible: bool,
    pub input_value: String,
    pub has_error: bool,
}

impl GotoPageState {
    pub fn open(&mut self, current_page: usize) {
        self.is_visible = true;
        // User-facing 1-based index
        self.input_value = (current_page + 1).to_string();
        self.has_error = false;
    }

    pub fn close(&mut self) {
        self.is_visible = false;
        self.input_value.clear();
        self.has_error = false;
    }

    pub fn set_input(&mut self, value: String) {
        self.input_value = value;
        self.has_error = false;
    }

    /// Validates the input string against total page count (1..=total_pages).
    /// Returns 0-based page index if valid.
    pub fn parse_target_page(&self, total_pages: usize) -> Option<usize> {
        if total_pages == 0 {
            return None;
        }
        let parsed = self.input_value.trim().parse::<usize>().ok()?;
        if parsed >= 1 && parsed <= total_pages {
            Some(parsed - 1)
        } else {
            None
        }
    }
}

/// Floating popover container style matching design tokens.
pub fn goto_dialog_style(theme: &iced::Theme) -> container::Style {
    let p = crate::ui::theme::color::BaseColors::palette(theme);
    let overlay = crate::ui::theme::AppTheme::from(theme).palette().overlay;

    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: iced::Border {
            color: overlay.floating_border,
            width: 1.0,
            radius: radius::LG.into(),
        },
        shadow: crate::ui::theme::tokens::elevation::floating_pill(overlay.floating_shadow),
        snap: false,
    }
}

/// Renders the floating Go to Page dialog popover.
pub fn view_goto_dialog<'a>(
    state: &'a GotoPageState,
    total_pages: usize,
    _font_size: f32,
) -> Element<'a, Message> {
    let placeholder = format!("1–{total_pages}");

    let input_field = text_input(&placeholder, &state.input_value)
        .id(GOTO_PAGE_INPUT_ID)
        .on_input(|val| Message::Pdf(PdfMsg::GotoPageInputChanged(val)))
        .on_submit(Message::Pdf(PdfMsg::GotoPageSubmit))
        .size(typography::BODY_MD)
        .style(default_text_input)
        .width(Length::Fixed(72.0))
        .padding([spacing::XS, spacing::S]);

    let label = text("Go to page:")
        .size(typography::BODY_MD)
        .align_y(Alignment::Center);

    let total_label = text(format!("/ {total_pages}"))
        .size(typography::BODY_MD)
        .align_y(Alignment::Center);

    let go_btn = button(
        text("Go")
            .size(typography::BODY_MD)
            .align_y(Alignment::Center),
    )
    .on_press(Message::Pdf(PdfMsg::GotoPageSubmit))
    .style(default_button)
    .padding([spacing::XS, spacing::M]);

    let close_btn = button(
        text("✕")
            .size(typography::BODY_MD)
            .align_y(Alignment::Center),
    )
    .on_press(Message::Pdf(PdfMsg::GotoPageClose))
    .style(default_button)
    .padding([spacing::XS, spacing::S]);

    let mut row_content = row![label, input_field, total_label, go_btn, close_btn]
        .spacing(spacing::S)
        .align_y(Alignment::Center);

    if state.has_error {
        let err_text = text("Invalid page")
            .size(typography::CAPTION)
            .style(|theme: &iced::Theme| {
                let role = crate::ui::theme::color::roles::palette(theme);
                text::Style {
                    color: Some(role.danger),
                }
            })
            .align_y(Alignment::Center);
        row_content = row_content.push(err_text);
    }

    let popover = container(row_content)
        .padding([spacing::S, spacing::M])
        .style(goto_dialog_style);

    // Overlay layer spanning Length::Fill with top-center alignment
    let overlay_container = container(popover)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Top)
        .padding(Padding {
            top: spacing::L,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        });

    iced::widget::opaque(overlay_container)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_page_state_lifecycle() {
        let mut state = GotoPageState::default();
        assert!(!state.is_visible);
        assert_eq!(state.input_value, "");

        state.open(4);
        assert!(state.is_visible);
        assert_eq!(state.input_value, "5");
        assert!(!state.has_error);

        state.set_input("12".to_string());
        assert_eq!(state.input_value, "12");

        let target = state.parse_target_page(20);
        assert_eq!(target, Some(11));

        let out_of_bounds = state.parse_target_page(10);
        assert_eq!(out_of_bounds, None);

        state.close();
        assert!(!state.is_visible);
        assert_eq!(state.input_value, "");
    }

    #[test]
    fn test_parse_target_page_edge_cases() {
        let mut state = GotoPageState::default();
        state.set_input("0".to_string());
        assert_eq!(state.parse_target_page(10), None);

        state.set_input("1".to_string());
        assert_eq!(state.parse_target_page(10), Some(0));

        state.set_input("10".to_string());
        assert_eq!(state.parse_target_page(10), Some(9));

        state.set_input("11".to_string());
        assert_eq!(state.parse_target_page(10), None);

        state.set_input("  5  ".to_string());
        assert_eq!(state.parse_target_page(10), Some(4));

        state.set_input("abc".to_string());
        assert_eq!(state.parse_target_page(10), None);

        state.set_input("1".to_string());
        assert_eq!(state.parse_target_page(0), None);
    }
}
