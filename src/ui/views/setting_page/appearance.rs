use iced::widget::{button, column, container, pick_list, row, slider, text};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::{ConfigManager, UiConfig};
use crate::ui::theme::color::{AppTheme, BaseColors};
use crate::ui::theme::default_row_button;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{default_card, default_pick_list, default_slider};

pub const AVAILABLE_THEMES: [&str; 8] = [
    "Dark",
    "Light",
    "Catppuccin Mocha",
    "Catppuccin Latte",
    "Tokyo Night",
    "Gruvbox Dark",
    "Nord",
    "Dracula",
];

const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 0.5;

const MIN_READER_WIDTH: f32 = 400.0;
const MAX_READER_WIDTH: f32 = 1600.0;
const READER_WIDTH_STEP: f32 = 20.0;
const DEFAULT_MAX_READER_WIDTH: f32 = 820.0;

const SLIDER_WIDTH: f32 = 170.0;
const PICKER_WIDTH: f32 = 240.0;

pub fn appearance_tab<'a>(
    theme: &Theme,
    config: &UiConfig,
    available_fonts: &'a [String],
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);

    // --- CARD 1: THEMES ---
    let theme_card = build_theme_card(theme, config, main_font);

    // --- CARD 2: TYPOGRAPHY ---
    let font_size = config.font_size;
    let size_control = row![
        slider(MIN_FONT_SIZE..=MAX_FONT_SIZE, font_size, |size| {
            Message::Settings(SettingsMsg::FontSizeChanged(size))
        })
        .step(FONT_SIZE_STEP)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{font_size:.1} px"))
            .size(11)
            .font(main_font)
            .width(58.0)
            .align_x(iced::alignment::Horizontal::Right)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let size_row = build_setting_row(
        "Base Font Size",
        "Default reading text scale for Markdown, EPUB and source code",
        size_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let sample_preview = container(
        text("The quick brown fox jumps over the lazy dog — 0123456789")
            .size(font_size.clamp(10.0, 24.0))
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text_dim),
            }),
    )
    .padding([spacing::XS, spacing::M])
    .width(iced::Length::Fill)
    .style(default_card);

    let main_font_picker = pick_list(available_fonts, config.font_family.clone(), |font| {
        Message::Settings(SettingsMsg::FontFamilySelected(font))
    })
    .placeholder("Default System Font")
    .font(main_font)
    .width(PICKER_WIDTH)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    let main_font_row = build_setting_row(
        "Main Font Family",
        "System proportional typeface for headings and interface reading",
        main_font_picker,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let mono_picker = pick_list(available_fonts, config.font_family_mono.clone(), |font| {
        Message::Settings(SettingsMsg::FontFamilyMonoSelected(font))
    })
    .placeholder("Default Monospace Font")
    .font(main_font)
    .width(PICKER_WIDTH)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    let mono_row = build_setting_row(
        "Monospace Font Family",
        "Fixed-pitch font for syntax highlighting and plain text viewers",
        mono_picker,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let epub_picker = pick_list(available_fonts, config.epub_font_family.clone(), |font| {
        Message::Settings(SettingsMsg::EpubFontFamilySelected(font))
    })
    .placeholder("Default Reader Font")
    .font(main_font)
    .width(PICKER_WIDTH)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    let epub_font_row = build_setting_row(
        "EPUB Reader Font",
        "Book typography font for digitized publications and articles",
        epub_picker,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let reader_width = config.max_text_width.unwrap_or(DEFAULT_MAX_READER_WIDTH);
    let width_control = row![
        slider(MIN_READER_WIDTH..=MAX_READER_WIDTH, reader_width, |val| {
            Message::Settings(SettingsMsg::MaxTextWidthChanged(Some(val)))
        })
        .step(READER_WIDTH_STEP)
        .width(SLIDER_WIDTH)
        .style({
            let theme = theme.clone();
            move |_, status| default_slider(&theme, status)
        }),
        text(format!("{reader_width:.0} px"))
            .size(11)
            .font(main_font)
            .width(58.0)
            .align_x(iced::alignment::Horizontal::Right)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
    ]
    .align_y(Alignment::Center)
    .spacing(spacing::S);

    let width_row = build_setting_row(
        "Max Text Reader Width",
        "Column width boundary for comfortable long-form document reading",
        width_control,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let typo_card = container(
        column![
            text("TYPOGRAPHY & READING COMFORT")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            size_row,
            sample_preview,
            main_font_row,
            mono_row,
            epub_font_row,
            width_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    column![theme_card, typo_card].spacing(spacing::M).into()
}

fn build_theme_card<'a>(theme: &Theme, config: &UiConfig, main_font: Font) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let current_theme = config.theme.as_ref().map(|t| {
        let app_theme = ConfigManager::resolve_theme(t);
        app_theme.display_name()
    });

    let header = text("VISUAL THEME")
        .size(11)
        .font(main_font)
        .style(move |_| iced::widget::text::Style {
            color: Some(base_colors.text_dim),
        });

    let swatch_dot = |c: iced::Color| {
        container(iced::widget::Space::new().width(11).height(11)).style(move |_| {
            container::Style {
                background: Some(c.into()),
                border: iced::Border {
                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.18),
                    width: 1.0,
                    radius: 5.5.into(),
                },
                ..Default::default()
            }
        })
    };

    let mut row1 = row![].spacing(spacing::S);
    let mut row2 = row![].spacing(spacing::S);

    for (idx, &theme_name) in AVAILABLE_THEMES.iter().enumerate() {
        let app_theme = theme_name.parse::<AppTheme>().unwrap_or(AppTheme::Dark);
        let palette = app_theme.palette();
        let is_active = current_theme == Some(theme_name);

        let swatches = row![
            swatch_dot(palette.base.bg),
            swatch_dot(palette.base.surface),
            swatch_dot(palette.roles.accent),
            swatch_dot(palette.base.text),
        ]
        .spacing(3);

        let theme_str = theme_name.to_string();
        let btn = button(
            column![swatches, text(theme_name).size(11).font(main_font)]
                .align_x(Alignment::Center)
                .spacing(spacing::XS),
        )
        .on_press(Message::Settings(SettingsMsg::ThemeChanged(theme_str)))
        .style({
            let theme_clone = theme.clone();
            move |_, status| default_row_button(&theme_clone, status, is_active)
        })
        .padding([spacing::XS, spacing::S])
        .width(iced::Length::Fill);

        if idx < 4 {
            row1 = row1.push(btn);
        } else {
            row2 = row2.push(btn);
        }
    }

    container(column![header, row1, row2].spacing(spacing::M))
        .padding(spacing::M)
        .style(default_card)
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
