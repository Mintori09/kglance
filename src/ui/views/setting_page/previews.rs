use iced::widget::{column, container, pick_list, row, text, toggler};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::{EpubReadingMode, UiConfig};
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{default_card, default_pick_list, default_toggler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpubModeDisplay {
    SingleChapter,
    Continuous,
}

impl std::fmt::Display for EpubModeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpubModeDisplay::SingleChapter => write!(f, "Single Chapter (Paged)"),
            EpubModeDisplay::Continuous => write!(f, "Continuous Scroll"),
        }
    }
}

pub fn previews_tab<'a>(theme: &Theme, config: &UiConfig, main_font: Font) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let theme_clone = theme.clone();

    // --- CARD 1: DOCUMENTS & BOOKS ---
    let current_mode = match config.epub_reading_mode {
        EpubReadingMode::SingleChapter => EpubModeDisplay::SingleChapter,
        EpubReadingMode::Continuous => EpubModeDisplay::Continuous,
    };

    let modes = vec![EpubModeDisplay::SingleChapter, EpubModeDisplay::Continuous];

    let epub_picker = pick_list(modes, Some(current_mode), |selected| {
        let mode = match selected {
            EpubModeDisplay::SingleChapter => EpubReadingMode::SingleChapter,
            EpubModeDisplay::Continuous => EpubReadingMode::Continuous,
        };
        Message::Settings(SettingsMsg::EpubReadingModeChanged(mode))
    })
    .font(main_font)
    .width(240)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    let epub_row = build_setting_row(
        "EPUB Reading Mode",
        "Choose between continuous smooth scroll and single chapter pagination",
        epub_picker,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let mermaid_cb = toggler(config.prefer_mermaid_cli)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::PreferMermaidCliChanged(enabled)))
        .style({
            let theme = theme_clone.clone();
            move |_, status| default_toggler(&theme, status)
        });

    let mermaid_row = build_setting_row(
        "Mermaid Diagram CLI",
        "Use external mmdc binary when installed for complex architecture diagrams",
        mermaid_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let doc_card = container(
        column![
            text("DOCUMENTS & E-BOOKS")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            epub_row,
            mermaid_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    // --- CARD 2: SOURCE CODE & DATA ---
    let wrap_cb = toggler(config.word_wrap)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::WordWrapChanged(enabled)))
        .style({
            let theme = theme_clone.clone();
            move |_, status| default_toggler(&theme, status)
        });

    let wrap_row = build_setting_row(
        "Code Word Wrap (Ctrl+W)",
        "Automatically soft-wrap long lines in source text and syntax viewers",
        wrap_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let json_cb = toggler(config.json_tree_view)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::JsonTreeViewChanged(enabled)))
        .style(move |_, status| default_toggler(&theme_clone, status));

    let json_row = build_setting_row(
        "JSON Tree View by Default",
        "Parse and display JSON documents in interactive collapsible tree mode",
        json_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let code_card = container(
        column![
            text("SOURCE CODE & DATA")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            wrap_row,
            json_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    column![doc_card, code_card].spacing(spacing::M).into()
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
