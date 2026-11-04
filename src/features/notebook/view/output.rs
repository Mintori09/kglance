use crate::app::Message;
use crate::features::notebook::types::CellOutput;
use crate::ui::theme::AppTheme;
use crate::ui::theme::font::get_code_font;
use iced::widget::{column, container, image, text};
use iced::{Element, Length, Padding};

pub fn render_cell_output<'a>(
    output: &'a CellOutput,
    execution_count: Option<i64>,
    font_size: f32,
    theme: AppTheme,
    font_family_mono: Option<&'a str>,
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
        CellOutput::Image { data, .. } => {
            let handle = image::Handle::from_bytes(data.clone());
            let img_widget = image(handle);

            let prompt_text = if let Some(ec) = execution_count {
                format!("Out [{ec}]:")
            } else {
                "Out:".to_string()
            };

            let prompt_badge = text(prompt_text)
                .font(code_font)
                .size((font_size * 0.82).max(11.0))
                .color(base.text_dim);

            let img_container = container(img_widget).padding(Padding::from([8, 12])).style(
                move |_: &iced::Theme| container::Style {
                    background: Some(base.surface.into()),
                    border: iced::Border {
                        color: base.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
            );

            column![prompt_badge, img_container].spacing(4).into()
        }
    }
}
