use iced::alignment::{Horizontal, Vertical};
use iced::highlighter::Theme as HighlightTheme;
use iced::widget::text_editor::{Action, Content};
use iced::widget::{container, row, text, text_editor};
use iced::{Alignment, Element, Font, Length};

use crate::app::Message;
use crate::ui::theme::default_text_editor;

const CHARACTER_WIDTH_RATIO: f32 = 0.65;
const GUTTER_PADDING: f32 = 16.0;
const GUTTER_SPACING: f32 = 6.0;

fn calculate_digit_count(line_count: usize) -> usize {
    if line_count > 0 {
        (line_count as f32).log10() as usize + 1
    } else {
        1
    }
}

pub fn line_number_width(line_count: usize, font_size: f32) -> Length {
    let digit_count = calculate_digit_count(line_count);
    let calculated_width =
        (digit_count as f32) * font_size * CHARACTER_WIDTH_RATIO + GUTTER_PADDING;
    Length::Fixed(calculated_width)
}

fn select_highlight_syntax(extension: &str) -> String {
    let ext = extension.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "rs" | "rust" => "rs".to_string(),
        "py" | "pyw" | "python" => "py".to_string(),
        "js" | "mjs" | "cjs" | "javascript" => "js".to_string(),
        "ts" | "mts" | "cts" | "typescript" => "ts".to_string(),
        "jsx" => "jsx".to_string(),
        "tsx" => "tsx".to_string(),
        "json" | "jsonc" => "json".to_string(),
        "html" | "htm" => "html".to_string(),
        "css" => "css".to_string(),
        "scss" | "sass" => "scss".to_string(),
        "md" | "markdown" => "md".to_string(),
        "toml" => "toml".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        "xml" | "svg" => "xml".to_string(),
        "sh" | "bash" | "zsh" => "sh".to_string(),
        "c" | "h" => "c".to_string(),
        "cpp" | "hpp" | "cc" | "cxx" => "cpp".to_string(),
        "java" => "java".to_string(),
        "go" => "go".to_string(),
        "sql" => "sql".to_string(),
        _ => ext,
    }
}

use crate::ui::theme::AppTheme;

fn select_highlight_theme(theme: AppTheme) -> HighlightTheme {
    if theme.is_dark() {
        HighlightTheme::Base16Mocha
    } else {
        HighlightTheme::InspiredGitHub
    }
}

fn generate_line_numbers_text(total_lines: usize) -> String {
    (1..=total_lines)
        .map(|number| number.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn code_editor<'a>(
    content: &'a Content,
    extension: &str,
    theme: AppTheme,
    font_size: f32,
    font: Font,
    on_action: fn(Action) -> Message,
) -> Element<'a, Message> {
    let highlight_theme = select_highlight_theme(theme);
    let line_count = content.text().lines().count().max(1);

    let line_numbers = generate_line_numbers_text(line_count);
    let gutter_width = line_number_width(line_count, font_size);
    let gutter_color = theme.palette().base.text_dim;

    let line_numbers_widget = container(
        text(line_numbers)
            .font(font)
            .size(font_size)
            .width(gutter_width)
            .color(gutter_color)
            .align_x(Horizontal::Right),
    )
    .padding(iced::Padding {
        top: 5.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    });

    let syntax = select_highlight_syntax(extension);

    let editor_widget = text_editor(content)
        .highlight(&syntax, highlight_theme)
        .font(font)
        .size(font_size)
        .wrapping(iced::widget::text::Wrapping::None)
        .on_action(on_action)
        .style(default_text_editor);

    row![
        container(line_numbers_widget).align_y(Vertical::Top),
        editor_widget
    ]
    .spacing(GUTTER_SPACING)
    .align_y(Alignment::Start)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_highlight_syntax() {
        assert_eq!(select_highlight_syntax("rs"), "rs");
        assert_eq!(select_highlight_syntax(".rs"), "rs");
        assert_eq!(select_highlight_syntax("PY"), "py");
        assert_eq!(select_highlight_syntax(".mjs"), "js");
        assert_eq!(select_highlight_syntax("jsonc"), "json");
        assert_eq!(select_highlight_syntax("unknown_ext"), "unknown_ext");
    }

    #[test]
    fn test_calculate_digit_count() {
        assert_eq!(calculate_digit_count(0), 1);
        assert_eq!(calculate_digit_count(1), 1);
        assert_eq!(calculate_digit_count(9), 1);
        assert_eq!(calculate_digit_count(10), 2);
        assert_eq!(calculate_digit_count(99), 2);
        assert_eq!(calculate_digit_count(100), 3);
        assert_eq!(calculate_digit_count(999), 3);
        assert_eq!(calculate_digit_count(1000), 4);
    }

    #[test]
    fn test_generate_line_numbers_text() {
        assert_eq!(generate_line_numbers_text(1), "1");
        assert_eq!(generate_line_numbers_text(3), "1\n2\n3");
    }
}
