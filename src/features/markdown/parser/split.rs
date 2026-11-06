use crate::parsers::markdown::{Block, Inline};

pub(crate) fn split_inlines_by_display_math(inlines: Vec<Inline>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    let flush_current = |current: &mut Vec<Inline>, blocks: &mut Vec<Block>| {
        if !current.is_empty() {
            let has_text = current.iter().any(|i| match i {
                Inline::Text(t) => !t.trim().is_empty(),
                Inline::SoftBreak => false,
                _ => true,
            });
            if has_text {
                blocks.push(Block::Paragraph(std::mem::take(current)));
            } else {
                current.clear();
            }
        }
    };

    for inline in inlines {
        match inline {
            Inline::DisplayMath(latex) => {
                flush_current(&mut current, &mut blocks);
                blocks.push(Block::Math(latex));
            }
            _ => {
                current.push(inline);
            }
        }
    }

    flush_current(&mut current, &mut blocks);
    blocks
}

pub(crate) fn split_inlines(inlines: Vec<Inline>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    let flush_current = |current: &mut Vec<Inline>, blocks: &mut Vec<Block>| {
        if !current.is_empty() {
            let has_text = current.iter().any(|i| match i {
                Inline::Text(t) => !t.trim().is_empty(),
                Inline::SoftBreak => false,
                _ => true,
            });
            if has_text {
                blocks.push(Block::Paragraph(std::mem::take(current)));
            } else {
                current.clear();
            }
        }
    };

    for inline in inlines {
        match inline {
            Inline::DisplayMath(latex) => {
                flush_current(&mut current, &mut blocks);
                blocks.push(Block::Math(latex));
            }
            Inline::Image {
                alt,
                url,
                width,
                height,
            } => {
                flush_current(&mut current, &mut blocks);
                blocks.push(Block::Image {
                    alt,
                    path: url,
                    link_url: None,
                    width,
                    height,
                });
            }
            Inline::Link { mut text, url }
                if text.len() == 1 && matches!(&text[0], Inline::Image { .. }) =>
            {
                flush_current(&mut current, &mut blocks);
                if let Some(Inline::Image {
                    alt,
                    url: img_url,
                    width,
                    height,
                }) = text.pop()
                {
                    blocks.push(Block::Image {
                        alt,
                        path: img_url,
                        link_url: Some(url),
                        width,
                        height,
                    });
                }
            }
            _ => {
                current.push(inline);
            }
        }
    }

    flush_current(&mut current, &mut blocks);
    blocks
}
