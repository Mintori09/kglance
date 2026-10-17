use iced::widget::{button, column, container, pick_list, row, text, text_input, toggler};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::{CustomActionEditorMsg, Message, SettingsMsg};
use crate::core::config::{EpubReadingMode, RubyDisplayMode, UiConfig};
use crate::core::selection_action::{CustomActionEditorState, SelectionMenuConfig};
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{
    default_button, default_card, default_inset, default_pick_list, default_text_input,
    default_toggler,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RubyModeDisplay {
    Stacked,
    Tooltip,
    Parentheses,
    Hidden,
}

impl std::fmt::Display for RubyModeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RubyModeDisplay::Stacked => write!(f, "Stacked (Furigana on top)"),
            RubyModeDisplay::Tooltip => write!(f, "Tooltip (Hover over text)"),
            RubyModeDisplay::Parentheses => write!(f, "Parentheses (Text(Ruby))"),
            RubyModeDisplay::Hidden => write!(f, "Hidden (Base text only)"),
        }
    }
}

pub fn previews_tab<'a>(
    theme: &Theme,
    config: &UiConfig,
    selection_menu_config: &'a SelectionMenuConfig,
    action_editor: Option<&'a CustomActionEditorState>,
    trash_config: &'a crate::core::config::TrashConfig,
    keybinding_config: &'a crate::core::config::KeybindingConfig,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let theme_clone = theme.clone();

    // --- CARD 1: DOCUMENTS & E-BOOKS ---
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

    let current_ruby_mode = match config.ruby_display_mode {
        RubyDisplayMode::Stacked => RubyModeDisplay::Stacked,
        RubyDisplayMode::Tooltip => RubyModeDisplay::Tooltip,
        RubyDisplayMode::Parentheses => RubyModeDisplay::Parentheses,
        RubyDisplayMode::Hidden => RubyModeDisplay::Hidden,
    };

    let ruby_modes = vec![
        RubyModeDisplay::Stacked,
        RubyModeDisplay::Tooltip,
        RubyModeDisplay::Parentheses,
        RubyModeDisplay::Hidden,
    ];

    let ruby_picker = pick_list(ruby_modes, Some(current_ruby_mode), |selected| {
        let mode = match selected {
            RubyModeDisplay::Stacked => RubyDisplayMode::Stacked,
            RubyModeDisplay::Tooltip => RubyDisplayMode::Tooltip,
            RubyModeDisplay::Parentheses => RubyDisplayMode::Parentheses,
            RubyModeDisplay::Hidden => RubyDisplayMode::Hidden,
        };
        Message::Settings(SettingsMsg::RubyDisplayModeChanged(mode))
    })
    .font(main_font)
    .width(240)
    .style({
        let theme = theme.clone();
        move |_, status| default_pick_list(&theme, status)
    });

    let ruby_row = build_setting_row(
        "Furigana / Ruby Text",
        "Display mode for Japanese readings and annotations (<ruby> tags)",
        ruby_picker,
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
            ruby_row,
            mermaid_row,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    // --- CARD 2: SELECTION & QUICK ACTIONS ---
    let selection_menu_cb = toggler(selection_menu_config.enabled)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::SelectionMenuToggled(enabled)))
        .style({
            let theme = theme_clone.clone();
            move |_, status| default_toggler(&theme, status)
        });

    let selection_menu_row = build_setting_row(
        "Selection Context Menu",
        "Show floating quick actions (copy, search, translate, custom scripts) on text selection",
        selection_menu_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let mut selection_card_items: Vec<Element<'a, Message>> = vec![
        text("SELECTION & QUICK ACTIONS")
            .size(11)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(base_colors.text_dim),
            })
            .into(),
        selection_menu_row,
    ];

    if selection_menu_config.enabled {
        let mut actions_column = column![].spacing(spacing::S);

        if selection_menu_config.custom_actions.is_empty() {
            actions_column = actions_column.push(
                text("No custom actions configured.")
                    .size(12)
                    .font(main_font)
                    .style(move |_| iced::widget::text::Style {
                        color: Some(base_colors.text_dim),
                    }),
            );
        } else {
            for (i, action) in selection_menu_config.custom_actions.iter().enumerate() {
                let action_enabled = action.enabled;
                let toggle_btn = toggler(action_enabled)
                    .on_toggle(move |enabled| {
                        Message::Settings(SettingsMsg::CustomAction(
                            CustomActionEditorMsg::ToggleEnabled { index: i, enabled },
                        ))
                    })
                    .style({
                        let theme = theme_clone.clone();
                        move |_, status| default_toggler(&theme, status)
                    });

                let mut badges = Vec::new();
                if action.popup {
                    badges.push("Popup");
                }
                if action.stream {
                    badges.push("Stream");
                }
                if action.toast {
                    badges.push("Toast");
                }
                let badge_str = if badges.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", badges.join(", "))
                };

                let name_label = row![
                    text(&action.name).size(13).font(main_font).style(move |_| {
                        iced::widget::text::Style {
                            color: Some(base_colors.text),
                        }
                    }),
                    text(badge_str).size(11).font(main_font).style(move |_| {
                        iced::widget::text::Style {
                            color: Some(base_colors.text_dim),
                        }
                    }),
                ]
                .spacing(spacing::XS)
                .align_y(Alignment::Center);

                let cmd_label = text(&action.command)
                    .size(11)
                    .font(main_font)
                    .style(move |_| iced::widget::text::Style {
                        color: Some(base_colors.text_dim),
                    });

                let info_col = column![name_label, cmd_label]
                    .spacing(spacing::XXS)
                    .width(iced::Length::Fill);

                let edit_btn = button(text("Edit").size(11).font(main_font))
                    .on_press(Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::OpenEdit(i),
                    )))
                    .style(default_button)
                    .padding([spacing::XXS, spacing::S]);

                let delete_btn = button(text("✕").size(11).font(main_font))
                    .on_press(Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::Delete(i),
                    )))
                    .style(default_button)
                    .padding([spacing::XXS, spacing::XS]);

                let action_row = row![toggle_btn, info_col, edit_btn, delete_btn]
                    .spacing(spacing::S)
                    .align_y(Alignment::Center);

                actions_column = actions_column.push(action_row);
            }
        }

        // Form editor if active
        if let Some(editor) = action_editor {
            let form_title = if editor.editing_index.is_some() {
                "Edit Custom Action"
            } else {
                "New Custom Action"
            };

            let theme_input_name = theme.clone();
            let name_input = text_input("Action Name (e.g. Translate VI)", &editor.name)
                .font(main_font)
                .padding([spacing::XXS, spacing::XS])
                .on_input(|s| {
                    Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::NameChanged(s),
                    ))
                })
                .style(move |_, status| default_text_input(&theme_input_name, status));

            let theme_input_cmd = theme.clone();
            let cmd_input = text_input("Shell Command (e.g. trans -b :vi)", &editor.command)
                .font(main_font)
                .padding([spacing::XXS, spacing::XS])
                .on_input(|s| {
                    Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::CommandChanged(s),
                    ))
                })
                .style(move |_, status| default_text_input(&theme_input_cmd, status));

            let popup_toggle = toggler(editor.popup)
                .on_toggle(|b| {
                    Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::PopupToggled(b),
                    ))
                })
                .style({
                    let theme = theme_clone.clone();
                    move |_, status| default_toggler(&theme, status)
                });
            let popup_opt = row![
                popup_toggle,
                text("Popup Card").size(11).font(main_font).style(move |_| {
                    iced::widget::text::Style {
                        color: Some(base_colors.text),
                    }
                })
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center);

            let stream_toggle = toggler(editor.stream)
                .on_toggle(|b| {
                    Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::StreamToggled(b),
                    ))
                })
                .style({
                    let theme = theme_clone.clone();
                    move |_, status| default_toggler(&theme, status)
                });
            let stream_opt = row![
                stream_toggle,
                text("Streaming").size(11).font(main_font).style(move |_| {
                    iced::widget::text::Style {
                        color: Some(base_colors.text),
                    }
                })
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center);

            let toast_toggle = toggler(editor.toast)
                .on_toggle(|b| {
                    Message::Settings(SettingsMsg::CustomAction(
                        CustomActionEditorMsg::ToastToggled(b),
                    ))
                })
                .style({
                    let theme = theme_clone.clone();
                    move |_, status| default_toggler(&theme, status)
                });
            let toast_opt = row![
                toast_toggle,
                text("Toast Notification")
                    .size(11)
                    .font(main_font)
                    .style(move |_| iced::widget::text::Style {
                        color: Some(base_colors.text)
                    })
            ]
            .spacing(spacing::XS)
            .align_y(Alignment::Center);

            let options_row = row![popup_opt, stream_opt, toast_opt]
                .spacing(spacing::M)
                .align_y(Alignment::Center);

            let cancel_btn = button(text("Cancel").size(11).font(main_font))
                .on_press(Message::Settings(SettingsMsg::CustomAction(
                    CustomActionEditorMsg::Cancel,
                )))
                .style(default_button)
                .padding([spacing::XXS, spacing::M]);

            let save_btn = button(text("Save Action").size(11).font(main_font))
                .on_press(Message::Settings(SettingsMsg::CustomAction(
                    CustomActionEditorMsg::Save,
                )))
                .style(default_button)
                .padding([spacing::XXS, spacing::M]);

            let form_box = container(
                column![
                    text(form_title)
                        .size(12)
                        .font(main_font)
                        .style(move |_| iced::widget::text::Style {
                            color: Some(base_colors.text),
                        }),
                    name_input,
                    cmd_input,
                    text("Selected text is passed via stdin and $KGLANCE_TEXT. File path is in $KGLANCE_FILE.")
                        .size(10)
                        .font(main_font)
                        .style(move |_| iced::widget::text::Style {
                            color: Some(base_colors.text_dim),
                        }),
                    options_row,
                    row![
                        iced::widget::Space::new().width(iced::Length::Fill),
                        cancel_btn,
                        save_btn,
                    ]
                    .spacing(spacing::S)
                    .align_y(Alignment::Center),
                ]
                .spacing(spacing::S),
            )
            .padding(spacing::M)
            .style(default_inset);

            actions_column = actions_column.push(form_box);
        } else {
            let add_btn = button(text("+ Add Custom Action").size(11).font(main_font))
                .on_press(Message::Settings(SettingsMsg::CustomAction(
                    CustomActionEditorMsg::OpenAdd,
                )))
                .style(default_button)
                .padding([spacing::XXS, spacing::M]);

            actions_column = actions_column.push(add_btn);
        }

        selection_card_items.push(actions_column.into());
    }

    let selection_card = container(column(selection_card_items).spacing(spacing::M))
        .padding(spacing::M)
        .style(default_card);

    // --- CARD 3: SOURCE CODE & DATA ---
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

    let theme_for_json = theme_clone.clone();
    let json_cb = toggler(config.json_tree_view)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::JsonTreeViewChanged(enabled)))
        .style(move |_, status| default_toggler(&theme_for_json, status));

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

    // --- CARD 4: TRASH & FILE ACTIONS ---
    let trash_enabled_cb = toggler(trash_config.enabled)
        .on_toggle(|enabled| Message::Settings(SettingsMsg::TrashEnabledChanged(enabled)))
        .style({
            let theme = theme_clone.clone();
            move |_, status| default_toggler(&theme, status)
        });

    let trash_enabled_row = build_setting_row(
        "Enable Move to Trash",
        "Allow sending active preview file to Trash via shortcut or menu",
        trash_enabled_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let confirm_cb = toggler(trash_config.confirm_on_delete)
        .on_toggle(|confirm| Message::Settings(SettingsMsg::TrashConfirmOnDeleteChanged(confirm)))
        .style({
            let theme = theme_clone.clone();
            move |_, status| default_toggler(&theme, status)
        });

    let confirm_row = build_setting_row(
        "Confirm Before Moving to Trash",
        "Prompt with a confirmation dialog before trashing the file",
        confirm_cb,
        main_font,
        base_colors.text,
        base_colors.text_dim,
    );

    let undo_label = text(format!(
        "Shortcut: {} • Undo: {} (Toast: {}s)",
        keybinding_config.trash_file,
        keybinding_config.undo_trash,
        trash_config.undo_timeout_seconds
    ))
    .size(11)
    .font(main_font)
    .style(move |_| iced::widget::text::Style {
        color: Some(base_colors.text_dim),
    });

    let trash_card = container(
        column![
            text("TRASH & FILE MANAGEMENT")
                .size(11)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(base_colors.text_dim)
                }),
            trash_enabled_row,
            confirm_row,
            undo_label,
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    column![doc_card, selection_card, code_card, trash_card]
        .spacing(spacing::M)
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
