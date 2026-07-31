use iced::widget::{column, container, pick_list, row, slider, text, text_input};
use iced::{Alignment, Element, Theme};
use std::process::Command;

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::UiConfig;
use crate::ui::theme::{default_pick_list, default_slider, default_text_input};

const TITLE_FONT_SIZE: f32 = 20.0;
const SECTION_FONT_SIZE: f32 = 14.0;
const SECTION_ITEM_SPACING: f32 = 4.0;
const SETTINGS_COLUMN_SPACING: f32 = 16.0;
const DIMENSION_ROW_SPACING: f32 = 8.0;
const CONTAINER_PADDING: f32 = 16.0;

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 0.5;

const MIN_READER_WIDTH: f32 = 400.0;
const MAX_READER_WIDTH: f32 = 1600.0;
const READER_WIDTH_STEP: f32 = 20.0;
const DEFAULT_MAX_READER_WIDTH: f32 = 820.0;

const FALLBACK_DEFAULT_WIDTH: u32 = 1024;
const FALLBACK_DEFAULT_HEIGHT: u32 = 768;
const FALLBACK_MIN_WIDTH: u32 = 800;
const FALLBACK_MIN_HEIGHT: u32 = 600;

const AVAILABLE_THEMES: [&str; 10] = [
    "Auto",
    "Dark",
    "Light",
    "Nord",
    "Solarized Dark",
    "Solarized Light",
    "Gruvbox Dark",
    "Catppuccin Mocha",
    "Dracula",
    "Tokyo Night",
];

pub fn settings_page<'a>(
    _theme: &Theme,
    config: &'a UiConfig,
    available_fonts: &'a [String],
) -> Element<'a, Message> {
    let settings_content = column![
        text("Application Settings").size(TITLE_FONT_SIZE),
        build_theme_section(config),
        build_font_size_section(config),
        build_font_picker_section(
            "Main Font Family",
            config.font_family.clone(),
            available_fonts,
            "Default System Font",
            |font| Message::Settings(SettingsMsg::FontFamilySelected(font))
        ),
        build_font_picker_section(
            "Monospace Font Family",
            config.font_family_mono.clone(),
            available_fonts,
            "Default Monospace Font",
            |font| Message::Settings(SettingsMsg::FontFamilyMonoSelected(font))
        ),
        build_font_picker_section(
            "EPUB Reader Font",
            config.epub_font_family.clone(),
            available_fonts,
            "Default Reader Font",
            |font| Message::Settings(SettingsMsg::EpubFontFamilySelected(font))
        ),
        build_reader_width_section(config),
        build_dimension_input_section(
            "Default Window Size (Width × Height)",
            config.default_width,
            config.default_height,
            FALLBACK_DEFAULT_WIDTH,
            FALLBACK_DEFAULT_HEIGHT,
            |w| Message::Settings(SettingsMsg::DefaultWidthChanged(w)),
            |h| Message::Settings(SettingsMsg::DefaultHeightChanged(h))
        ),
        build_dimension_input_section(
            "Minimum Window Size (Width × Height)",
            config.min_width,
            config.min_height,
            FALLBACK_MIN_WIDTH,
            FALLBACK_MIN_HEIGHT,
            |w| Message::Settings(SettingsMsg::MinWidthChanged(w)),
            |h| Message::Settings(SettingsMsg::MinHeightChanged(h))
        ),
    ]
    .spacing(SETTINGS_COLUMN_SPACING);

    container(settings_content)
        .padding(CONTAINER_PADDING)
        .into()
}

pub fn get_system_fonts() -> Vec<String> {
    let output = Command::new("fc-match")
        .args(["-a", "-f", "%{family}\n"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut fonts: Vec<String> = stdout
                .lines()
                .flat_map(|line| line.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            fonts.sort();
            fonts.dedup();
            fonts
        }
        _ => vec!["Sans-Serif".to_string()],
    }
}

fn build_theme_section<'a>(config: &'a UiConfig) -> Element<'a, Message> {
    let themes: Vec<String> = AVAILABLE_THEMES.iter().map(|&t| t.to_string()).collect();
    let theme_picker = pick_list(themes, config.theme.clone(), |theme| {
        Message::Settings(SettingsMsg::ThemeChanged(theme))
    })
    .placeholder("System Default (Auto)")
    .style(default_pick_list);

    column![text("Color Theme").size(SECTION_FONT_SIZE), theme_picker]
        .spacing(SECTION_ITEM_SPACING)
        .into()
}

fn build_font_size_section<'a>(config: &'a UiConfig) -> Element<'a, Message> {
    let font_size = config.font_size;
    let label = text(format!("Font Size: {:.1} px", font_size)).size(SECTION_FONT_SIZE);
    let slider_widget = slider(MIN_FONT_SIZE..=MAX_FONT_SIZE, font_size, |size| {
        Message::Settings(SettingsMsg::FontSizeChanged(size))
    })
    .step(FONT_SIZE_STEP)
    .style(default_slider);

    column![label, slider_widget]
        .spacing(SECTION_ITEM_SPACING)
        .into()
}

fn build_font_picker_section<'a, F>(
    label_text: &'static str,
    selected_font: Option<String>,
    available_fonts: &'a [String],
    placeholder: &'static str,
    on_select: F,
) -> Element<'a, Message>
where
    F: 'static + Fn(String) -> Message,
{
    let picker = pick_list(available_fonts, selected_font, on_select)
        .placeholder(placeholder)
        .style(default_pick_list);

    column![text(label_text).size(SECTION_FONT_SIZE), picker]
        .spacing(SECTION_ITEM_SPACING)
        .into()
}

fn build_reader_width_section<'a>(config: &'a UiConfig) -> Element<'a, Message> {
    let width = config.max_text_width.unwrap_or(DEFAULT_MAX_READER_WIDTH);
    let label = text(format!("Max Text Reader Width: {:.0} px", width)).size(SECTION_FONT_SIZE);
    let slider_widget = slider(MIN_READER_WIDTH..=MAX_READER_WIDTH, width, |val| {
        Message::Settings(SettingsMsg::MaxTextWidthChanged(Some(val)))
    })
    .step(READER_WIDTH_STEP)
    .style(default_slider);

    column![label, slider_widget]
        .spacing(SECTION_ITEM_SPACING)
        .into()
}

fn build_dimension_input_section<'a, FW, FH>(
    label_text: &'static str,
    current_width: u32,
    current_height: u32,
    fallback_width: u32,
    fallback_height: u32,
    on_width_change: FW,
    on_height_change: FH,
) -> Element<'a, Message>
where
    FW: 'static + Fn(u32) -> Message,
    FH: 'static + Fn(u32) -> Message,
{
    let width_str = current_width.to_string();
    let height_str = current_height.to_string();

    let width_input = text_input("Width", &width_str)
        .on_input(move |input| {
            let parsed_val = input.parse::<u32>().unwrap_or(fallback_width);
            on_width_change(parsed_val)
        })
        .style(default_text_input);

    let height_input = text_input("Height", &height_str)
        .on_input(move |input| {
            let parsed_val = input.parse::<u32>().unwrap_or(fallback_height);
            on_height_change(parsed_val)
        })
        .style(default_text_input);

    column![
        text(label_text).size(SECTION_FONT_SIZE),
        row![width_input, text("×").size(SECTION_FONT_SIZE), height_input]
            .spacing(DIMENSION_ROW_SPACING)
            .align_y(Alignment::Center)
    ]
    .spacing(SECTION_ITEM_SPACING)
    .into()
}
