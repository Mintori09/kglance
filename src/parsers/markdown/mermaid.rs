use std::sync::{Arc, LazyLock};

use super::Block;
use super::MarkdownParser;
use crate::{log_debug, log_error};

static FONT_DB: LazyLock<Arc<resvg::usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    Arc::new(fontdb)
});

impl MarkdownParser {
    pub fn render_mermaid_blocks(blocks: &mut [Block]) {
        for block in blocks {
            if let Block::Mermaid { lines, rendered } = block
                && rendered.is_none()
            {
                let code = lines.join("\n");
                *rendered = render_mermaid_to_png(&code, None);
            }
        }
    }
}

pub fn render_mermaid_to_png(
    code: &str,
    bg_color: Option<resvg::tiny_skia::Color>,
) -> Option<Vec<u8>> {
    log_debug!("Rendering Mermaid diagram ({} bytes)", code.len());

    let options = mermaid_rs_renderer::RenderOptions::modern();
    let mut svg = mermaid_rs_renderer::render_with_options(code, options).ok()?;

    if let Some(svg_start) = svg.find("<svg")
        && let Some(rect_idx) = svg[svg_start..].find("<rect")
    {
        let abs_rect_idx = svg_start + rect_idx;
        if let Some(rect_end) = svg[abs_rect_idx..].find('>') {
            let rect_tag = &svg[abs_rect_idx..abs_rect_idx + rect_end];
            if let Some(fill_pos) = rect_tag
                .find("fill=\"#FFFFFF\"")
                .or_else(|| rect_tag.find("fill=\"#ffffff\""))
                .or_else(|| rect_tag.find("fill=\"white\""))
            {
                let fill_len = if rect_tag[fill_pos..].starts_with("fill=\"white\"") {
                    12
                } else {
                    14
                };
                svg.replace_range(
                    abs_rect_idx + fill_pos..abs_rect_idx + fill_pos + fill_len,
                    "fill=\"none\"",
                );
            }
        }
    }

    let opt = resvg::usvg::Options {
        fontdb: FONT_DB.clone(),
        ..Default::default()
    };

    let rtree = resvg::usvg::Tree::from_str(&svg, &opt).ok()?;

    let pixmap_size = rtree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

    if let Some(bg) = bg_color {
        pixmap.fill(bg);
    }

    resvg::render(
        &rtree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let png_data = pixmap.encode_png().ok()?;

    if png_data.is_empty() {
        log_error!("Generated PNG is empty");
        return None;
    }

    log_debug!("Mermaid render succeeded ({} bytes PNG)", png_data.len());

    Some(png_data)
}
