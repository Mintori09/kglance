use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::UiConfig;
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{default_button, default_card, default_row_button, default_text_input};

pub fn window_tab<'a>(theme: &Theme, config: &UiConfig, main_font: Font) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);

    // --- CARD 1: LAUNCH GEOMETRY ---
    let default_size_inputs = build_dimension_inputs(
        theme,
        config.default_width,
        config.default_height,
        main_font,
        base_colors.text_dim,
        |w| Message::Settings(SettingsMsg::DefaultWidthChanged(w)),
        |h| Message::Settings(SettingsMsg::DefaultHeightChanged(h)),
    );

    let default_row = build_setting_row(
        "Default Window Size",
        "Target dimensions when opening preview without saved geometry",
        default_size_inputs,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let preset_btn = |label: &'static str, w: u32, h: u32| {
        let is_active = config.default_width == w && config.default_height == h;
        let theme_clone = theme.clone();
        button(text(label).size(11).font(main_font))
            .on_press(Message::Settings(SettingsMsg::SetWindowPreset {
                width: w,
                height: h,
            }))
            .style(move |_, status| default_row_button(&theme_clone, status, is_active))
            .padding([spacing::XXS, spacing::S])
    };

    let presets_row = row![
        preset_btn("800×600", 800, 600),
        preset_btn("1024×768", 1024, 768),
        preset_btn("1280×720", 1280, 720),
        preset_btn("1440×900", 1440, 900),
    ]
    .spacing(spacing::XS);

    let use_current_btn = button(text("Use Current Window Size").size(11).font(main_font))
        .on_press(Message::Settings(SettingsMsg::UseCurrentWindowSize))
        .style(default_button)
        .padding([spacing::XS, spacing::S]);

    let presets_control = column![presets_row, use_current_btn].spacing(spacing::XS);

    let presets_setting_row = build_setting_row(
        "Quick Presets & Capture",
        "Apply standard desktop aspect ratios or capture active window size",
        presets_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let launch_card = container(
        column![
            text("DEFAULT LAUNCH GEOMETRY")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            default_row,
            presets_setting_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    // --- CARD 2: CONSTRAINTS ---
    let min_size_inputs = build_dimension_inputs(
        theme,
        config.min_width,
        config.min_height,
        main_font,
        base_colors.text_dim,
        |w| Message::Settings(SettingsMsg::MinWidthChanged(w)),
        |h| Message::Settings(SettingsMsg::MinHeightChanged(h)),
    );

    let min_row = build_setting_row(
        "Minimum Window Size",
        "Hard floor boundaries preventing UI elements from being clipped",
        min_size_inputs,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let constraints_card = container(
        column![
            text("SIZE BOUNDARIES & CONSTRAINTS")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            min_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    column![launch_card, constraints_card]
        .spacing(spacing::M)
        .into()
}

fn build_dimension_inputs<'a, FW, FH>(
    theme: &Theme,
    current_width: u32,
    current_height: u32,
    main_font: Font,
    dim_color: iced::Color,
    on_width_change: FW,
    on_height_change: FH,
) -> Element<'a, Message>
where
    FW: 'static + Fn(u32) -> Message,
    FH: 'static + Fn(u32) -> Message,
{
    let width_str = current_width.to_string();
    let height_str = current_height.to_string();

    let theme_w = theme.clone();
    let width_input = text_input("Width", &width_str)
        .font(main_font)
        .width(65)
        .on_input(move |input| match input.parse::<u32>() {
            Ok(parsed_val) if parsed_val > 0 => on_width_change(parsed_val),
            _ => Message::None,
        })
        .style(move |_, status| default_text_input(&theme_w, status));

    let theme_h = theme.clone();
    let height_input = text_input("Height", &height_str)
        .font(main_font)
        .width(65)
        .on_input(move |input| match input.parse::<u32>() {
            Ok(parsed_val) if parsed_val > 0 => on_height_change(parsed_val),
            _ => Message::None,
        })
        .style(move |_, status| default_text_input(&theme_h, status));

    row![
        width_input,
        text("×")
            .size(13)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(dim_color)
            }),
        height_input,
        text("px")
            .size(11)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(dim_color)
            }),
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center)
    .into()
}

fn build_setting_row<'a>(
    title: &'static str,
    desc: &'static str,
    control: impl Into<Element<'a, Message>>,
    main_font: Font,
    text_color: iced::Color,
    dim_color: iced::Color,
) -> Element<'a, Message> {
    row![
        column![
            text(title)
                .size(13)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(text_color)
                }),
            text(desc)
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(dim_color)
                }),
        ]
        .spacing(spacing::XXS)
        .width(iced::Length::Fill),
        container(control)
            .width(250.0)
            .align_x(iced::alignment::Horizontal::Right),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::M)
    .into()
}
