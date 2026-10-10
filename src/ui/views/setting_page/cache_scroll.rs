use iced::widget::{column, container, row, slider, text};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::{CacheConfig, ScrollConfigOptions};
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{default_card, default_checkbox, default_slider};

const MIN_MEMORY_MB: usize = 128;
const MAX_MEMORY_MB: usize = 2048;
const STEP_MEMORY_MB: usize = 128;

const MIN_DISK_MB: usize = 256;
const MAX_DISK_MB: usize = 4096;
const STEP_DISK_MB: usize = 256;

const MIN_FRICTION: f32 = 0.5;
const MAX_FRICTION: f32 = 5.0;
const STEP_FRICTION: f32 = 0.1;

const MIN_SPRING: f32 = 50.0;
const MAX_SPRING: f32 = 400.0;
const STEP_SPRING: f32 = 10.0;

const SLIDER_WIDTH: f32 = 180.0;

pub fn cache_scroll_tab<'a>(
    theme: &Theme,
    cache_config: &CacheConfig,
    scroll_config: &ScrollConfigOptions,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let theme_clone = theme.clone();

    // --- CARD 1: STORAGE & MEMORY CACHE ---
    let mem_mb = cache_config.max_memory_mb;
    let mem_control = row![
        slider(
            (MIN_MEMORY_MB as f32)..=(MAX_MEMORY_MB as f32),
            mem_mb as f32,
            |val| Message::Settings(SettingsMsg::MaxMemoryMbChanged(val as usize)),
        )
        .step(STEP_MEMORY_MB as f32)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{mem_mb} MB"))
            .size(11)
            .font(main_font)
            .width(62)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let mem_row = build_setting_row(
        "Memory Cache (RAM)",
        "Ceiling for in-memory parsed documents and decoded image bitmaps",
        mem_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let disk_mb = cache_config.max_disk_cache_mb;
    let disk_control = row![
        slider(
            (MIN_DISK_MB as f32)..=(MAX_DISK_MB as f32),
            disk_mb as f32,
            |val| Message::Settings(SettingsMsg::MaxDiskCacheMbChanged(val as usize)),
        )
        .step(STEP_DISK_MB as f32)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{disk_mb} MB"))
            .size(11)
            .font(main_font)
            .width(62)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let disk_row = build_setting_row(
        "Disk Cache Storage",
        "Storage limit for persistent PDF pages and thumbnails in ~/.cache/kglance",
        disk_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let cache_card = container(
        column![
            text("STORAGE & MEMORY LIMITS")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            mem_row,
            disk_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    // --- CARD 2: SMOOTH SCROLLING PHYSICS ---
    let smooth_cb = iced::widget::checkbox(scroll_config.smooth_scroll_enabled)
        .label("")
        .on_toggle(|enabled| Message::Settings(SettingsMsg::SmoothScrollChanged(enabled)))
        .style(move |_, status| default_checkbox(&theme_clone, status));

    let smooth_row = build_setting_row(
        "Kinetic Smooth Scrolling",
        "Enable high-framerate interpolated scrolling for touchpad & mouse wheel",
        smooth_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let friction = scroll_config.friction;
    let friction_control = row![
        slider(MIN_FRICTION..=MAX_FRICTION, friction, |val| {
            Message::Settings(SettingsMsg::ScrollFrictionChanged(val))
        })
        .step(STEP_FRICTION)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{friction:.1}"))
            .size(11)
            .font(main_font)
            .width(45)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let friction_row = build_setting_row(
        "Kinetic Scroll Friction",
        "Deceleration rate for inertia glide gestures (lower = longer slide)",
        friction_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let spring = scroll_config.spring_stiffness;
    let spring_control = row![
        slider(MIN_SPRING..=MAX_SPRING, spring, |val| {
            Message::Settings(SettingsMsg::ScrollSpringStiffnessChanged(val))
        })
        .step(STEP_SPRING)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{spring:.0}"))
            .size(11)
            .font(main_font)
            .width(45)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let spring_row = build_setting_row(
        "Spring Stiffness",
        "Rubber-band bounce resistance coefficient at viewport boundaries",
        spring_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let scroll_card = container(
        column![
            text("SMOOTH SCROLLING PHYSICS")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            smooth_row,
            friction_row,
            spring_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    column![cache_card, scroll_card].spacing(spacing::M).into()
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
        control.into()
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::M)
    .into()
}
