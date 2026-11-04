use crate::app::Message;
use crate::features::notebook::types::CellOutput;
use crate::ui::theme::AppTheme;
use crate::ui::theme::font::get_code_font;
use iced::widget::{button, column, container, image, text};
use iced::{Element, Length, Padding};

pub fn render_cell_output<'a>(
    output: &'a CellOutput,
    execution_count: Option<i64>,
    font_size: f32,
    theme: AppTheme,
    font_family_mono: Option<&'a str>,
    cell_index: usize,
    output_index: usize,
) -> Element<'a, Message> {
    let base = theme.palette().base;
    let roles = theme.palette().roles;
    let code_font = get_code_font(font_family_mono);

    match output {
        CellOutput::Text {
            text: out_text,
            is_error,
            truncated,
        } => {
            let text_color = if *is_error { roles.danger } else { base.text };
            let bg_color = if *is_error {
                iced::Color {
                    a: 0.12,
                    ..roles.danger
                }
            } else {
                base.surface
            };
            let border_color = if *is_error { roles.danger } else { base.border };

            let prompt_text = if let Some(ec) = execution_count {
                format!("Out [{ec}]:")
            } else {
                "Out:".to_string()
            };

            let prompt_badge = text(prompt_text)
                .font(code_font)
                .size((font_size * 0.82).max(11.0))
                .color(base.text_dim);

            let mut content_col = column![
                text(out_text.as_str())
                    .font(code_font)
                    .size((font_size * 0.9).max(12.0))
                    .color(text_color)
            ]
            .spacing(4);

            if *truncated {
                let notice = text("[Output truncated at 200 lines...]")
                    .font(code_font)
                    .size((font_size * 0.8).max(11.0))
                    .color(roles.link);
                content_col = content_col.push(notice);
            }

            let text_container = container(content_col)
                .padding(Padding::from([8, 12]))
                .width(Length::Fill)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                });

            column![prompt_badge, text_container].spacing(4).into()
        }
        CellOutput::Image { handle, .. } => {
            let img_widget = image(handle.clone()).content_fit(iced::ContentFit::Contain);

            let prompt_text = if let Some(ec) = execution_count {
                format!("Out [{ec}]:")
            } else {
                "Out:".to_string()
            };

            let prompt_badge = text(prompt_text)
                .font(code_font)
                .size((font_size * 0.82).max(11.0))
                .color(base.text_dim);

            let img_container = container(img_widget)
                .padding(Padding::from([8, 12]))
                .width(Length::Fill)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(base.surface.into()),
                    border: iced::Border {
                        color: base.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                });

            let image_id = format!("cell_{cell_index}_out_{output_index}");
            let img_btn = button(img_container)
                .on_press(crate::app::messages::NotebookMsg::OpenImageLightbox(image_id).into())
                .padding(0)
                .style(|_: &iced::Theme, status: button::Status| {
                    let border_color = match status {
                        button::Status::Hovered => iced::Color::from_rgba(0.4, 0.7, 1.0, 0.5),
                        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.7, 1.0, 0.8),
                        _ => iced::Color::TRANSPARENT,
                    };
                    button::Style {
                        background: None,
                        text_color: iced::Color::TRANSPARENT,
                        border: iced::Border {
                            width: 1.0,
                            color: border_color,
                            radius: 6.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    }
                });

            column![prompt_badge, img_btn].spacing(4).into()
        }
    }
}
