use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::app::messages::PdfMsg;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::sidebar_entry_style;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::{spacing, typography};

/// Represents an item in the PDF outline/table-of-contents tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfOutlineNode {
    /// Title of the outline section.
    pub title: String,
    /// 0-based target page index.
    pub page: usize,
    /// Hierarchy depth (0-based for indentation calculation).
    pub depth: usize,
    /// Whether this node's children are currently visible in the sidebar.
    pub is_expanded: bool,
    /// Child outline nodes, or None if leaf node.
    pub children: Option<Vec<PdfOutlineNode>>,
}

/// State tracking for the PDF Table of Contents sidebar.
#[derive(Debug, Clone, Default)]
pub struct PdfOutlineState {
    /// Hierarchical tree nodes of the outline.
    pub nodes: Vec<PdfOutlineNode>,
}

impl PdfOutlineState {
    /// Construct outline state from a hierarchical list of nodes.
    pub fn new(nodes: Vec<PdfOutlineNode>) -> Self {
        Self { nodes }
    }

    /// Construct outline state from flat `PdfTocEntry` items, reconstructing tree hierarchy.
    pub fn from_toc_entries(entries: &[crate::features::pdf::parser::PdfTocEntry]) -> Self {
        if entries.is_empty() {
            return Self::default();
        }

        // Reconstruct hierarchy from level values (1-based level)
        let nodes = build_tree_from_toc_entries(entries);
        Self { nodes }
    }

    /// Toggle expand/collapse state for node matching target_page and target_title.
    pub fn toggle_expanded(&mut self, target_page: usize, target_title: &str) {
        Self::toggle_in_nodes(&mut self.nodes, target_page, target_title);
    }

    fn toggle_in_nodes(
        nodes: &mut [PdfOutlineNode],
        target_page: usize,
        target_title: &str,
    ) -> bool {
        for node in nodes.iter_mut() {
            if node.page == target_page && node.title == target_title && node.children.is_some() {
                node.is_expanded = !node.is_expanded;
                return true;
            }
            if let Some(ref mut children) = node.children
                && Self::toggle_in_nodes(children, target_page, target_title)
            {
                return true;
            }
        }
        false
    }
}

/// Converts a slice of `PdfTocEntry` (which have level: 1, 2, 3...) into nested `PdfOutlineNode`.
pub fn build_tree_from_toc_entries(
    entries: &[crate::features::pdf::parser::PdfTocEntry],
) -> Vec<PdfOutlineNode> {
    fn parse_level(
        entries: &[crate::features::pdf::parser::PdfTocEntry],
        idx: &mut usize,
        current_depth: usize,
    ) -> Vec<PdfOutlineNode> {
        let mut result = Vec::new();

        while *idx < entries.len() {
            let entry = &entries[*idx];
            let node_depth = (entry.level.saturating_sub(1)) as usize;

            if node_depth < current_depth {
                // Return to parent level
                break;
            }

            if node_depth == current_depth {
                let title = entry.title.clone();
                let page = entry.page;
                *idx += 1;

                // Check if subsequent entries are deeper (children)
                let children = if *idx < entries.len()
                    && (entries[*idx].level.saturating_sub(1) as usize) > current_depth
                {
                    let child_nodes = parse_level(entries, idx, current_depth + 1);
                    if child_nodes.is_empty() {
                        None
                    } else {
                        Some(child_nodes)
                    }
                } else {
                    None
                };

                result.push(PdfOutlineNode {
                    title,
                    page,
                    depth: current_depth,
                    is_expanded: true,
                    children,
                });
            } else {
                // If node_depth > current_depth unexpectedly, parse at deeper depth
                let child_nodes = parse_level(entries, idx, current_depth + 1);
                result.extend(child_nodes);
            }
        }

        result
    }

    let mut idx = 0;
    parse_level(entries, &mut idx, 0)
}

/// Flatten hierarchical `mupdf::Outline` nodes into a standardized tree of `PdfOutlineNode`.
pub fn build_outline_tree(
    nodes: &[mupdf::Outline],
    doc: &mupdf::Document,
    depth: usize,
) -> Vec<PdfOutlineNode> {
    let mut tree = Vec::new();

    for node in nodes {
        let title = node.title.trim().to_string();
        if title.is_empty() {
            continue;
        }

        let page = resolve_outline_page(node, doc);

        let children = if !node.down.is_empty() {
            let child_nodes = build_outline_tree(&node.down, doc, depth + 1);
            if child_nodes.is_empty() {
                None
            } else {
                Some(child_nodes)
            }
        } else {
            None
        };

        tree.push(PdfOutlineNode {
            title,
            page,
            depth,
            is_expanded: true, // Expanded by default so users see initial structure
            children,
        });
    }

    tree
}

/// Resolve the 0-based target page for a given outline item.
pub fn resolve_outline_page(node: &mupdf::Outline, doc: &mupdf::Document) -> usize {
    if let Some(ref dest) = node.dest {
        return dest.loc.page_number as usize;
    }

    if let Some(ref uri) = node.uri {
        if let Ok(Some(dest)) = doc.resolve_link(uri) {
            return dest.loc.page_number as usize;
        } else if let Some(stripped) = uri.strip_prefix('#') {
            if let Ok(num) = stripped.parse::<usize>() {
                return num.saturating_sub(1);
            } else if let Some(page_str) = stripped.strip_prefix("page=") {
                return page_str.parse::<usize>().unwrap_or(1).saturating_sub(1);
            }
        }
    }

    0
}

/// Recursively collects all currently visible nodes in top-down display order.
pub fn collect_visible_outline_nodes<'a>(
    nodes: &'a [PdfOutlineNode],
    out: &mut Vec<&'a PdfOutlineNode>,
) {
    for node in nodes {
        out.push(node);
        if node.is_expanded
            && let Some(ref children) = node.children
        {
            collect_visible_outline_nodes(children, out);
        }
    }
}

/// Render the PDF Outline TOC list sidebar element.
pub fn render_pdf_outline<'a>(
    outline_state: &'a PdfOutlineState,
    current_page: usize,
    theme: AppTheme,
) -> Element<'a, Message> {
    if outline_state.nodes.is_empty() {
        return container(text("No TOC available").size(typography::BODY))
            .padding(spacing::M)
            .center_x(Length::Fill)
            .into();
    }

    let mut visible_nodes = Vec::new();
    collect_visible_outline_nodes(&outline_state.nodes, &mut visible_nodes);

    if visible_nodes.is_empty() {
        return container(text("No TOC available").size(typography::BODY))
            .padding(spacing::M)
            .center_x(Length::Fill)
            .into();
    }

    // Determine active item based on highest page <= current_page
    let active_node_idx = visible_nodes
        .iter()
        .rposition(|node| node.page <= current_page);

    let mut toc_col = column![].spacing(spacing::XXS).padding(spacing::XS + 2.0);

    for (idx, node) in visible_nodes.into_iter().enumerate() {
        let is_active = active_node_idx == Some(idx);
        let row_item = render_outline_entry(node, is_active, theme);
        toc_col = toc_col.push(row_item);
    }

    scroll_pane("pdf_toc_scroll", toc_col).build()
}

fn render_outline_entry<'a>(
    node: &'a PdfOutlineNode,
    is_active: bool,
    theme: AppTheme,
) -> Element<'a, Message> {
    let p = theme.palette().base;
    let indent = (node.depth as f32) * 12.0;

    let has_children = node.children.as_ref().is_some_and(|c| !c.is_empty());

    let expand_btn: Element<'a, Message> = if has_children {
        let arrow_char = if node.is_expanded { "▾" } else { "▸" };
        let arrow_color = theme.palette().sidebar.arrow_text;
        button(
            container(text(arrow_char).size(10).color(arrow_color))
                .width(Length::Fixed(14.0))
                .align_x(Alignment::Center),
        )
        .on_press(
            PdfMsg::ToggleOutlineNode {
                page: node.page,
                title: node.title.clone(),
            }
            .into(),
        )
        .padding(0)
        .style(move |_, _| button::Style {
            background: None,
            text_color: arrow_color,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        })
        .into()
    } else {
        Space::new().width(Length::Fixed(14.0)).into()
    };

    let title_text = text(&node.title).size(typography::BODY).width(Length::Fill);

    let page_label = text(format!("{}", node.page + 1))
        .size(typography::CAPTION)
        .style(move |_| iced::widget::text::Style {
            color: Some(p.text_dim),
        });

    let row_content = row![
        Space::new().width(Length::Fixed(indent)),
        expand_btn,
        title_text,
        page_label
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let btn = button(row_content)
        .on_press(PdfMsg::TocItemClicked(node.page).into())
        .width(Length::Fill)
        .style(move |iced_theme, status| sidebar_entry_style(iced_theme, status, is_active, theme))
        .padding([4, 6]);

    container(btn).width(Length::Fill).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::pdf::parser::PdfTocEntry;

    #[test]
    fn test_outline_node_expansion_toggle() {
        let mut state = PdfOutlineState {
            nodes: vec![
                PdfOutlineNode {
                    title: "Chapter 1".to_string(),
                    page: 0,
                    depth: 0,
                    is_expanded: true,
                    children: Some(vec![PdfOutlineNode {
                        title: "Section 1.1".to_string(),
                        page: 2,
                        depth: 1,
                        is_expanded: false,
                        children: None,
                    }]),
                },
                PdfOutlineNode {
                    title: "Chapter 2".to_string(),
                    page: 5,
                    depth: 0,
                    is_expanded: false,
                    children: None,
                },
            ],
        };

        {
            let mut visible = Vec::new();
            collect_visible_outline_nodes(&state.nodes, &mut visible);
            assert_eq!(visible.len(), 3);
        }

        // Toggle Chapter 1 closed
        state.toggle_expanded(0, "Chapter 1");
        {
            let mut visible = Vec::new();
            collect_visible_outline_nodes(&state.nodes, &mut visible);
            assert_eq!(visible.len(), 2);
            assert_eq!(visible[0].title, "Chapter 1");
            assert_eq!(visible[1].title, "Chapter 2");
        }

        // Toggle Chapter 1 open again
        state.toggle_expanded(0, "Chapter 1");
        {
            let mut visible = Vec::new();
            collect_visible_outline_nodes(&state.nodes, &mut visible);
            assert_eq!(visible.len(), 3);
        }
    }

    #[test]
    fn test_build_tree_from_toc_entries() {
        let entries = vec![
            PdfTocEntry {
                title: "Chapter 1".to_string(),
                page: 0,
                level: 1,
            },
            PdfTocEntry {
                title: "Section 1.1".to_string(),
                page: 1,
                level: 2,
            },
            PdfTocEntry {
                title: "Section 1.2".to_string(),
                page: 3,
                level: 2,
            },
            PdfTocEntry {
                title: "Chapter 2".to_string(),
                page: 10,
                level: 1,
            },
        ];

        let state = PdfOutlineState::from_toc_entries(&entries);
        assert_eq!(state.nodes.len(), 2);
        assert_eq!(state.nodes[0].title, "Chapter 1");
        assert_eq!(state.nodes[0].depth, 0);
        let ch1_children = state.nodes[0].children.as_ref().unwrap();
        assert_eq!(ch1_children.len(), 2);
        assert_eq!(ch1_children[0].title, "Section 1.1");
        assert_eq!(ch1_children[0].depth, 1);
        assert_eq!(ch1_children[1].title, "Section 1.2");
        assert_eq!(ch1_children[1].depth, 1);

        assert_eq!(state.nodes[1].title, "Chapter 2");
        assert!(state.nodes[1].children.is_none());
    }
}
