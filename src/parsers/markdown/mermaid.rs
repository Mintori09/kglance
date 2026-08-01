use std::fs;
use std::process::Command;
use std::sync::{Arc, LazyLock};
use tempfile::NamedTempFile;

use super::{Block, MarkdownParser};
use crate::{log_debug, log_error};

static GLOBAL_FONT_DATABASE: LazyLock<Arc<resvg::usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut font_database = resvg::usvg::fontdb::Database::new();
    font_database.load_system_fonts();
    Arc::new(font_database)
});

impl MarkdownParser {
    pub fn render_mermaid_blocks(blocks: &mut [Block], prefer_cli: bool) {
        for block in blocks {
            if let Block::Mermaid { lines, rendered } = block
                && rendered.is_none()
            {
                let diagram_code = lines.join("\n");
                *rendered = render_mermaid_to_png(&diagram_code, None, prefer_cli);
            }
        }
    }
}

pub fn render_mermaid_to_png(
    diagram_code: &str,
    background_color: Option<resvg::tiny_skia::Color>,
    prefer_cli: bool,
) -> Option<Vec<u8>> {
    if prefer_cli {
        if let Some(png) = render_mermaid_by_mermaid_cli(diagram_code, background_color) {
            return Some(png);
        }
        log_error!("mmdc rendering failed or not available, falling back to mmdr");
    }

    log_debug!("Rendering Mermaid diagram ({} bytes)", diagram_code.len());

    let render_options = mermaid_rs_renderer::RenderOptions::modern();
    let mut svg_content =
        mermaid_rs_renderer::render_with_options(diagram_code, render_options).ok()?;

    make_background_transparent(&mut svg_content);

    let render_tree = parse_svg_tree(&svg_content)?;
    let png_bytes = rasterize_svg_to_png(&render_tree, background_color)?;

    if png_bytes.is_empty() {
        log_error!("Generated PNG is empty");
        return None;
    }

    log_debug!("Mermaid render succeeded ({} bytes PNG)", png_bytes.len());

    Some(png_bytes)
}

fn color_to_hex(color: resvg::tiny_skia::Color) -> String {
    const COLOR_CHANNEL_MAX: f32 = 255.0;
    let red = (color.red() * COLOR_CHANNEL_MAX) as u8;
    let green = (color.green() * COLOR_CHANNEL_MAX) as u8;
    let blue = (color.blue() * COLOR_CHANNEL_MAX) as u8;
    let alpha = color.alpha();

    if alpha < 1.0 {
        let alpha_byte = (alpha * COLOR_CHANNEL_MAX) as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", red, green, blue, alpha_byte)
    } else {
        format!("#{:02x}{:02x}{:02x}", red, green, blue)
    }
}

fn build_mermaid_cli_command(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    background_color: Option<resvg::tiny_skia::Color>,
) -> Command {
    const MERMAID_CLI_BINARY: &str = "mmdc";
    const TRANSPARENT_COLOR: &str = "transparent";
    let mut command = Command::new(MERMAID_CLI_BINARY);
    command.arg("-i").arg(input_path).arg("-o").arg(output_path);

    let background = background_color
        .map(color_to_hex)
        .unwrap_or_else(|| TRANSPARENT_COLOR.to_string());

    command.arg("-b").arg(background);
    command
}

fn render_mermaid_by_mermaid_cli(
    diagram_code: &str,
    background_color: Option<resvg::tiny_skia::Color>,
) -> Option<Vec<u8>> {
    const INPUT_EXTENSION: &str = ".mmd";
    const OUTPUT_EXTENSION: &str = ".png";
    crate::log_info!("Attempting to render Mermaid diagram via mermaid-cli (mmdc)");

    let input_file = NamedTempFile::with_suffix(INPUT_EXTENSION).ok()?;
    fs::write(input_file.path(), diagram_code).ok()?;

    let output_file = NamedTempFile::with_suffix(OUTPUT_EXTENSION).ok()?;
    let output_path = output_file.path().to_path_buf();

    let mut command = build_mermaid_cli_command(input_file.path(), &output_path, background_color);
    let execution_status = command.status().ok()?;

    if !execution_status.success() {
        log_error!("Failed to render Mermaid diagram via mmdc CLI (exit code non-zero)");
        return None;
    }

    let bytes = fs::read(&output_path).ok()?;
    crate::log_info!(
        "Successfully rendered Mermaid via mmdc CLI ({} bytes)",
        bytes.len()
    );
    Some(bytes)
}

fn make_background_transparent(svg: &mut String) {
    let Some(svg_start) = svg.find("<svg") else {
        return;
    };
    let Some(rect_offset) = svg[svg_start..].find("<rect") else {
        return;
    };

    let rect_start = svg_start + rect_offset;
    let Some(rect_end) = svg[rect_start..].find('>') else {
        return;
    };

    let rect_tag = &svg[rect_start..rect_start + rect_end];
    let white_fill_patterns = ["fill=\"#FFFFFF\"", "fill=\"#ffffff\"", "fill=\"white\""];

    for fill_pattern in white_fill_patterns {
        if let Some(fill_pos) = rect_tag.find(fill_pattern) {
            let start = rect_start + fill_pos;
            let end = start + fill_pattern.len();
            svg.replace_range(start..end, "fill=\"none\"");
            break;
        }
    }
}

fn parse_svg_tree(svg_content: &str) -> Option<resvg::usvg::Tree> {
    let options = resvg::usvg::Options {
        fontdb: GLOBAL_FONT_DATABASE.clone(),
        ..Default::default()
    };

    resvg::usvg::Tree::from_str(svg_content, &options).ok()
}

fn rasterize_svg_to_png(
    render_tree: &resvg::usvg::Tree,
    background_color: Option<resvg::tiny_skia::Color>,
) -> Option<Vec<u8>> {
    let dimensions = render_tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(dimensions.width(), dimensions.height())?;

    if let Some(color) = background_color {
        pixmap.fill(color);
    }

    resvg::render(
        render_tree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use resvg::tiny_skia::Color;
    use std::path::Path;
    use std::process::Command;

    fn is_mmdc_installed() -> bool {
        Command::new("mmdc")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_color_to_hex_opaque() {
        let color = Color::from_rgba8(255, 0, 128, 255);
        assert_eq!(color_to_hex(color), "#ff0080");
    }

    #[test]
    fn test_color_to_hex_transparent() {
        let color = Color::from_rgba8(255, 0, 128, 127);
        assert_eq!(color_to_hex(color), "#ff00807f");
    }

    #[test]
    fn test_color_to_hex_black_and_white() {
        let white = Color::from_rgba8(255, 255, 255, 255);
        let black = Color::from_rgba8(0, 0, 0, 255);

        assert_eq!(color_to_hex(white), "#ffffff");
        assert_eq!(color_to_hex(black), "#000000");
    }

    #[test]
    fn test_build_mermaid_cli_command_with_color() {
        let input_path = Path::new("/tmp/input.mmd");
        let output_path = Path::new("/tmp/output.png");
        let bg_color = Color::from_rgba8(255, 255, 255, 255);

        let command = build_mermaid_cli_command(input_path, output_path, Some(bg_color));
        let program = command.get_program().to_string_lossy();
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(program, "mmdc");
        assert_eq!(
            args,
            vec![
                "-i",
                "/tmp/input.mmd",
                "-o",
                "/tmp/output.png",
                "-b",
                "#ffffff"
            ]
        );
    }

    #[test]
    fn test_build_mermaid_cli_command_without_color() {
        let input_path = Path::new("/tmp/input.mmd");
        let output_path = Path::new("/tmp/output.png");

        let command = build_mermaid_cli_command(input_path, output_path, None);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            args,
            vec![
                "-i",
                "/tmp/input.mmd",
                "-o",
                "/tmp/output.png",
                "-b",
                "transparent"
            ]
        );
    }

    #[test]
    fn test_render_mermaid_by_mermaid_cli() {
        if !is_mmdc_installed() {
            eprintln!("Skipping test: `mmdc` CLI is not installed.");
            return;
        }

        let diagram = "graph TD;\n    A-->B;";
        let result = render_mermaid_by_mermaid_cli(diagram, None);

        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_render_mermaid_to_png_prefer_cli_fallback() {
        // When prefer_cli is true, if mmdc is not available (or fails), it falls back to mmdr
        let diagram = "graph TD;\n    A-->B;";
        let result = render_mermaid_to_png(diagram, None, true);

        assert!(
            result.is_some(),
            "should succeed either via mmdc or fallback to mmdr"
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_render_mermaid_to_png_prefer_cli_false() {
        let diagram = "graph TD;\n    A-->B;";
        let result = render_mermaid_to_png(diagram, None, false);

        assert!(result.is_some(), "should succeed via mmdr");
        assert!(!result.unwrap().is_empty());
    }
}
