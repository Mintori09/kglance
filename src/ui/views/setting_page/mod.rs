use iced::widget::{button, column, container, pick_list, row, slider, text, text_input};
use iced::{Alignment, Element, Font, Theme};
use std::process::Command;
use std::sync::OnceLock;

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::UiConfig;
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::font::get_main_font;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{
    default_button, default_card, default_checkbox, default_pick_list, default_slider,
    default_text_input,
};

const TITLE_FONT_SIZE: f32 = 18.0;
const SECTION_FONT_SIZE: f32 = 13.0;

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

const AVAILABLE_THEMES: [&str; 8] = [
    "Dark",
    "Light",
    "Catppuccin Mocha",
    "Catppuccin Latte",
    "Tokyo Night",
    "Gruvbox Dark",
    "Nord",
    "Dracula",
];

pub fn settings_page<'a>(
    theme: &Theme,
    config: &UiConfig,
    available_fonts: &'a [String],
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let main_font = get_main_font(config.font_family.as_deref());

    let header_row = row![
        text("Application Settings")
            .size(TITLE_FONT_SIZE)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
        iced::widget::Space::new().width(iced::Length::Fill),
        button(text("✕").size(13).font(main_font))
            .on_press(crate::app::messages::NavigationMsg::ToggleSettingsClicked.into())
            .style(default_button)
            .padding([spacing::XS, spacing::S])
    ]
    .align_y(Alignment::Center);

    let settings_content = column![
        header_row,
        build_theme_section(theme, config, main_font),
        build_font_size_section(theme, config, main_font),
        build_font_picker_section(
            theme,
            "Main Font Family",
            config.font_family.clone(),
            available_fonts,
            "Default System Font",
            main_font,
            |font| Message::Settings(SettingsMsg::FontFamilySelected(font))
        ),
        build_font_picker_section(
            theme,
            "Monospace Font Family",
            config.font_family_mono.clone(),
            available_fonts,
            "Default Monospace Font",
            main_font,
            |font| Message::Settings(SettingsMsg::FontFamilyMonoSelected(font))
        ),
        build_font_picker_section(
            theme,
            "EPUB Reader Font",
            config.epub_font_family.clone(),
            available_fonts,
            "Default Reader Font",
            main_font,
            |font| Message::Settings(SettingsMsg::EpubFontFamilySelected(font))
        ),
        build_reader_width_section(theme, config, main_font),
        build_word_wrap_section(theme, config, main_font),
        build_json_tree_view_section(theme, config, main_font),
        build_dimension_input_section(
            theme,
            "Default Window Size (Width × Height)",
            config.default_width,
            config.default_height,
            FALLBACK_DEFAULT_WIDTH,
            FALLBACK_DEFAULT_HEIGHT,
            main_font,
            |w| Message::Settings(SettingsMsg::DefaultWidthChanged(w)),
            |h| Message::Settings(SettingsMsg::DefaultHeightChanged(h))
        ),
        build_dimension_input_section(
            theme,
            "Minimum Window Size (Width × Height)",
            config.min_width,
            config.min_height,
            FALLBACK_MIN_WIDTH,
            FALLBACK_MIN_HEIGHT,
            main_font,
            |w| Message::Settings(SettingsMsg::MinWidthChanged(w)),
            |h| Message::Settings(SettingsMsg::MinHeightChanged(h))
        ),
    ]
    .spacing(spacing::L);

    container(settings_content)
        .padding(spacing::L)
        .style(default_card)
        .into()
}

pub fn get_system_fonts() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
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
    })
}

fn build_theme_section<'a>(
    theme: &Theme,
    config: &UiConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let themes: Vec<String> = AVAILABLE_THEMES.iter().map(|&t| t.to_string()).collect();
    let current_theme = config.theme.as_ref().map(|t| {
        let app_theme = crate::core::config::ConfigManager::resolve_theme(t);
        app_theme.display_name().to_string()
    });

    let theme_picker = pick_list(themes, current_theme, |theme| {
        Message::Settings(SettingsMsg::ThemeChanged(theme))
    })
    .placeholder("Dark")
    .font(main_font)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    column![
        text("Color Theme")
            .size(SECTION_FONT_SIZE)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
        theme_picker
    ]
    .spacing(spacing::XS)
    .into()
}

fn build_font_size_section<'a>(
    theme: &Theme,
    config: &UiConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let font_size = config.font_size;
    let label = text(format!("Font Size: {font_size:.1} px"))
        .size(SECTION_FONT_SIZE)
        .font(main_font)
        .style(move |_| iced::widget::text::Style {
            color: Some(base_colors.text),
        });
    let slider_widget = slider(MIN_FONT_SIZE..=MAX_FONT_SIZE, font_size, |size| {
        Message::Settings(SettingsMsg::FontSizeChanged(size))
    })
    .step(FONT_SIZE_STEP)
    .style({
        let theme = theme.clone();
        move |_, status| default_slider(&theme, status)
    });

    column![label, slider_widget].spacing(spacing::XS).into()
}

fn build_font_picker_section<'a, F>(
    theme: &Theme,
    label_text: &'static str,
    selected_font: Option<String>,
    available_fonts: &'a [String],
    placeholder: &'static str,
    main_font: Font,
    on_select: F,
) -> Element<'a, Message>
where
    F: 'static + Fn(String) -> Message,
{
    let base_colors = BaseColors::palette(theme);
    let picker = pick_list(available_fonts, selected_font, on_select)
        .placeholder(placeholder)
        .font(main_font)
        .style({
            let theme = theme.clone();
            move |_, status| default_pick_list(&theme, status)
        });

    column![
        text(label_text)
            .size(SECTION_FONT_SIZE)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
        picker
    ]
    .spacing(spacing::XS)
    .into()
}

fn build_reader_width_section<'a>(
    theme: &Theme,
    config: &UiConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let width = config.max_text_width.unwrap_or(DEFAULT_MAX_READER_WIDTH);
    let label = text(format!("Max Text Reader Width: {width:.0} px"))
        .size(SECTION_FONT_SIZE)
        .font(main_font)
        .style(move |_| iced::widget::text::Style {
            color: Some(base_colors.text),
        });
    let slider_widget = slider(MIN_READER_WIDTH..=MAX_READER_WIDTH, width, |val| {
        Message::Settings(SettingsMsg::MaxTextWidthChanged(Some(val)))
    })
    .step(READER_WIDTH_STEP)
    .style({
        let theme = theme.clone();
        move |_, status| default_slider(&theme, status)
    });

    column![label, slider_widget].spacing(spacing::XS).into()
}

fn build_word_wrap_section<'a>(
    theme: &Theme,
    config: &UiConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let theme_clone = theme.clone();
    let checkbox_widget = iced::widget::checkbox(config.word_wrap)
        .label("Word Wrap in Code Previews (Ctrl+W)")
        .font(main_font)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::WordWrapChanged(enabled)))
        .text_size(SECTION_FONT_SIZE)
        .style(move |_, status| default_checkbox(&theme_clone, status));

    column![checkbox_widget].spacing(spacing::XS).into()
}

fn build_json_tree_view_section<'a>(
    theme: &Theme,
    config: &UiConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let theme_clone = theme.clone();
    let checkbox_widget = iced::widget::checkbox(config.json_tree_view)
        .label("JSON Tree View by Default")
        .font(main_font)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::JsonTreeViewChanged(enabled)))
        .text_size(SECTION_FONT_SIZE)
        .style(move |_, status| default_checkbox(&theme_clone, status));

    column![checkbox_widget].spacing(spacing::XS).into()
}

#[allow(clippy::too_many_arguments)]
fn build_dimension_input_section<'a, FW, FH>(
    theme: &Theme,
    label_text: &'static str,
    current_width: u32,
    current_height: u32,
    fallback_width: u32,
    fallback_height: u32,
    main_font: Font,
    on_width_change: FW,
    on_height_change: FH,
) -> Element<'a, Message>
where
    FW: 'static + Fn(u32) -> Message,
    FH: 'static + Fn(u32) -> Message,
{
    let base_colors = BaseColors::palette(theme);
    let width_str = current_width.to_string();
    let height_str = current_height.to_string();

    let theme_w = theme.clone();
    let width_input = text_input("Width", &width_str)
        .font(main_font)
        .on_input(move |input| {
            let parsed_val = input.parse::<u32>().unwrap_or(fallback_width);
            on_width_change(parsed_val)
        })
        .style(move |_, status| default_text_input(&theme_w, status));

    let theme_h = theme.clone();
    let height_input = text_input("Height", &height_str)
        .font(main_font)
        .on_input(move |input| {
            let parsed_val = input.parse::<u32>().unwrap_or(fallback_height);
            on_height_change(parsed_val)
        })
        .style(move |_, status| default_text_input(&theme_h, status));

    column![
        text(label_text)
            .size(SECTION_FONT_SIZE)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
        row![
            width_input,
            text("×")
                .size(SECTION_FONT_SIZE)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            height_input
        ]
        .spacing(spacing::S)
        .align_y(Alignment::Center)
    ]
    .spacing(spacing::XS)
    .into()
}
