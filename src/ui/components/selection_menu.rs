use iced::widget::{Space, button, column, container, mouse_area, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Theme};
use std::cell::Cell;

use crate::app::messages::{Message, SelectionMenuMsg};
use crate::core::KglanceState;
use crate::core::selection_action::SelectionMenuState;
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::color::roles;
use crate::ui::theme::font::get_main_font;
use crate::ui::theme::tokens::{radius, spacing, typography};
use crate::ui::theme::{default_card, default_raised};
use crate::ui::types::RenderContext;

fn toolbar_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);
    let (bg, text_color) = match status {
        button::Status::Hovered => (Some(role.accent_hover.into()), Color::WHITE),
        button::Status::Pressed => (Some(role.accent_pressed.into()), Color::WHITE),
        _ => (None, p.text),
    };
    button::Style {
        background: bg,
        text_color,
        border: Border {
            radius: radius::SM.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn close_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = BaseColors::palette(theme);
    let (bg, text_color) = match status {
        button::Status::Hovered => (
            Some(Color::from_rgba(1.0, 0.2, 0.2, 0.2).into()),
            Color::WHITE,
        ),
        button::Status::Pressed => (
            Some(Color::from_rgba(1.0, 0.2, 0.2, 0.4).into()),
            Color::WHITE,
        ),
        _ => (None, p.text_dim),
    };
    button::Style {
        background: bg,
        text_color,
        border: Border {
            radius: radius::SM.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn render_selection_menu_layer<'a>(state: &'a KglanceState) -> Option<Element<'a, Message>> {
    let pos = state.selection_menu.position()?;
    let main_font = get_main_font(state.font_family.as_deref());

    let (content_widget, menu_w, menu_h): (Element<'a, Message>, f32, f32) =
        match &state.selection_menu {
            SelectionMenuState::Hidden => return None,
            SelectionMenuState::Toolbar { selected_text, .. } => {
                let copy_btn: Element<'a, Message> =
                    button(text("Copy").size(typography::CAPTION).font(main_font))
                        .on_press(SelectionMenuMsg::CopyClicked(selected_text.clone()).into())
                        .style(toolbar_button_style)
                        .padding([spacing::XXS, spacing::XS])
                        .into();

                let mut buttons_row = row![copy_btn]
                    .spacing(spacing::XXS)
                    .align_y(Alignment::Center);

                for action in &state.selection_menu_config.custom_actions {
                    if !action.enabled {
                        continue;
                    }
                    let action_btn: Element<'a, Message> =
                        button(text(&action.name).size(typography::CAPTION).font(main_font))
                            .on_press(
                                SelectionMenuMsg::CustomActionClicked {
                                    action: action.clone(),
                                    text: selected_text.clone(),
                                }
                                .into(),
                            )
                            .style(toolbar_button_style)
                            .padding([spacing::XXS, spacing::XS])
                            .into();
                    buttons_row = buttons_row.push(action_btn);
                }

                let toolbar_box = container(buttons_row)
                    .padding([spacing::XXS, spacing::XS])
                    .style(default_raised);

                let win_w = state.current_window_size.width;
                let est_w = crate::core::selection_action::estimate_toolbar_width(
                    &state.selection_menu_config.custom_actions,
                )
                .min((win_w - 24.0).max(220.0));
                (
                    iced::widget::opaque(toolbar_box),
                    est_w,
                    crate::core::selection_action::TOOLBAR_HEIGHT,
                )
            }
            SelectionMenuState::Running { action_name, .. } => {
                let label = text(format!("Running {action_name}..."))
                    .size(typography::CAPTION)
                    .font(main_font);
                let pill = container(row![label].padding([spacing::XXS, spacing::M]))
                    .style(default_raised);
                (
                    iced::widget::opaque(pill),
                    crate::core::selection_action::RUNNING_WIDTH,
                    crate::core::selection_action::RUNNING_HEIGHT,
                )
            }
            SelectionMenuState::Result {
                action_name,
                content,
                blocks,
                is_streaming,
                ..
            } => {
                let title_text = text(action_name).size(typography::BODY_MD).font(main_font);

                let action_indicator: Element<'a, Message> = if *is_streaming {
                    text("...").size(typography::CAPTION).font(main_font).into()
                } else {
                    button(text("Copy").size(typography::CAPTION).font(main_font))
                        .on_press(SelectionMenuMsg::CopyClicked(content.clone()).into())
                        .style(toolbar_button_style)
                        .padding([spacing::XXS, spacing::XS])
                        .into()
                };

                let close_btn: Element<'a, Message> =
                    button(text("✕").size(typography::CAPTION).font(main_font))
                        .on_press(SelectionMenuMsg::Dismiss.into())
                        .style(close_button_style)
                        .padding([spacing::XXS, spacing::XS])
                        .into();

                let header = row![
                    title_text,
                    Space::new().width(Length::Fill),
                    action_indicator,
                    close_btn,
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center);

                let popup_font_size = state.font_size.clamp(13.0, 15.0);

                let counter = Cell::new(0);
                let block_ctx = RenderContext {
                    block_index: 0,
                    selection_range: None,
                    drag_active: false,
                    search_query: "",
                    active_match: 0,
                    counter: &counter,
                    theme: state.app_theme,
                    font_size: popup_font_size,
                    font_family: state.font_family.as_deref(),
                    font_family_mono: state.font_family_mono.as_deref(),
                    ruby_display_mode: state.ruby_display_mode,
                };

                let body_element: Element<'a, Message> = if blocks.is_empty() {
                    if *is_streaming && content.is_empty() {
                        text("Loading...").size(popup_font_size).into()
                    } else {
                        text(content.as_str()).size(popup_font_size).into()
                    }
                } else {
                    let mut block_elements: Vec<Element<'a, Message>> =
                        Vec::with_capacity(blocks.len());
                    for (i, block) in blocks.iter().enumerate() {
                        let el = crate::features::markdown::view::render_block(
                            i,
                            block,
                            &state.markdown,
                            &block_ctx,
                        );
                        block_elements.push(el);
                    }
                    column(block_elements).spacing(spacing::S).into()
                };

                let scrollable_body = scrollable(body_element)
                    .height(Length::Fixed(260.0))
                    .width(Length::Fill);

                let card_layout = column![header, scrollable_body].spacing(spacing::S);

                let card = container(card_layout)
                    .width(crate::core::selection_action::RESULT_WIDTH)
                    .padding(spacing::M)
                    .style(default_card);

                (
                    iced::widget::opaque(card),
                    crate::core::selection_action::RESULT_WIDTH,
                    crate::core::selection_action::RESULT_HEIGHT,
                )
            }
        };

    let win_w = state.current_window_size.width;
    let win_h = state.current_window_size.height;

    let clamped_x = (pos.x - 24.0).clamp(12.0, (win_w - menu_w - 12.0).max(12.0));
    let clamped_y = if pos.y - menu_h - 12.0 >= 44.0 {
        pos.y - menu_h - 12.0
    } else {
        (pos.y + 16.0).clamp(44.0, (win_h - menu_h - 12.0).max(44.0))
    };

    let positioned_menu = container(content_widget)
        .padding(Padding {
            top: clamped_y,
            left: clamped_x,
            bottom: 0.0,
            right: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill);

    if matches!(state.selection_menu, SelectionMenuState::Result { .. }) {
        let backdrop = mouse_area(
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(SelectionMenuMsg::Dismiss.into());

        let layer = iced::widget::Stack::new()
            .push(backdrop)
            .push(positioned_menu);

        Some(layer.into())
    } else {
        Some(positioned_menu.into())
    }
}
