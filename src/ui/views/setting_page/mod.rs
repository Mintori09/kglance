use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Theme};
use std::process::Command;
use std::sync::OnceLock;

use crate::app::messages::{Message, NavigationMsg, SettingsMsg};
use crate::core::SettingTab;
use crate::core::config::{CacheConfig, ScrollConfigOptions, UiConfig};
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::font::get_main_font;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{
    default_button, default_button_primary, default_card, default_inset, default_row_button,
};

pub mod appearance;
pub mod cache_scroll;
pub mod previews;
pub mod window;

pub use appearance::AVAILABLE_THEMES;

const TITLE_FONT_SIZE: f32 = 16.0;

pub fn settings_page<'a>(
    theme: &Theme,
    config: &UiConfig,
    cache_config: &CacheConfig,
    scroll_config: &ScrollConfigOptions,
    active_tab: SettingTab,
    available_fonts: &'a [String],
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let main_font = get_main_font(config.font_family.as_deref());

    // Header: Title + Close (✕)
    let header_row = row![
        text("Application Settings")
            .size(TITLE_FONT_SIZE)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text)
            }),
        iced::widget::Space::new().width(iced::Length::Fill),
        button(text("✕").size(13).font(main_font))
            .on_press(NavigationMsg::ToggleSettingsClicked.into())
            .style(default_button)
            .padding([spacing::XS, spacing::S])
    ]
    .align_y(Alignment::Center);

    // Segmented Tabs: [ 🎨 Appearance ] [ 🪟 Window ] [ 📑 Previews ] [ ⚡ Cache & Scroll ]
    let tab_button = |label: &'static str, tab: SettingTab| {
        let is_selected = active_tab == tab;
        button(
            text(label)
                .size(12)
                .font(main_font)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Settings(SettingsMsg::TabChanged(tab)))
        .style({
            let theme_clone = theme.clone();
            move |_, status| default_row_button(&theme_clone, status, is_selected)
        })
        .padding([spacing::XS, spacing::M])
        .width(iced::Length::Fill)
    };

    let segmented_tabs = container(
        row![
            tab_button("🎨 Appearance", SettingTab::Appearance),
            tab_button("🪟 Window", SettingTab::Window),
            tab_button("📑 Previews", SettingTab::Previews),
            tab_button("⚡ Cache & Scroll", SettingTab::CacheScroll),
        ]
        .spacing(spacing::XXS)
        .padding(spacing::XXS)
        .width(iced::Length::Fill),
    )
    .style(default_inset);

    // Active Tab Content
    let tab_content: Element<'a, Message> = match active_tab {
        SettingTab::Appearance => {
            appearance::appearance_tab(theme, config, available_fonts, main_font)
        }
        SettingTab::Window => window::window_tab(theme, config, main_font),
        SettingTab::Previews => previews::previews_tab(theme, config, main_font),
        SettingTab::CacheScroll => {
            cache_scroll::cache_scroll_tab(theme, cache_config, scroll_config, main_font)
        }
    };

    let scrollable_content = iced::widget::scrollable(tab_content).height(iced::Length::Fill);

    // Footer: Reset to Defaults + Done
    let footer_row = row![
        button(text("↺ Reset to Defaults").size(11).font(main_font))
            .on_press(Message::Settings(SettingsMsg::ResetToDefaults))
            .style(default_button)
            .padding([spacing::XS, spacing::M]),
        iced::widget::Space::new().width(iced::Length::Fill),
        button(text("Done").size(12).font(main_font))
            .on_press(NavigationMsg::ToggleSettingsClicked.into())
            .style(default_button_primary)
            .padding([spacing::XS, spacing::XL]),
    ]
    .align_y(Alignment::Center);

    let modal_layout =
        column![header_row, segmented_tabs, scrollable_content, footer_row,].spacing(spacing::M);

    container(modal_layout)
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
