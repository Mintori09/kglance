use iced::widget::{Space, button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Element, Length, Padding, Shadow};

use super::types::{FileTreeNode, FileTreeState};
use crate::app::Message;
use crate::app::messages::FileTreeMsg;
use crate::ui::components::sidebar::sidebar_entry_style;
use crate::ui::theme::tokens::{spacing, typography};
use crate::ui::theme::{AppTheme, icon_theme};

const HEADER_PADDING: Padding = Padding {
    top: 8.0,
    right: 8.0,
    bottom: 8.0,
    left: 10.0,
};

const ENTRY_PADDING: Padding = Padding {
    top: 3.0,
    right: 6.0,
    bottom: 3.0,
    left: 4.0,
};

pub fn view_file_tree<'a>(
    state: &'a FileTreeState,
    current_file_path: &'a std::path::Path,
    theme: AppTheme,
    font_family: Option<&str>,
) -> Element<'a, Message> {
    let main_font = crate::ui::theme::font::get_main_font(font_family);

    let header = render_header(state, theme, main_font);

    let mut visible_nodes = Vec::new();
    collect_visible_nodes(&state.nodes, &mut visible_nodes);

    let entries: Vec<Element<'a, Message>> = visible_nodes
        .into_iter()
        .map(|node| render_node_row(node, current_file_path, theme, main_font))
        .collect();

    let content_scroll = scrollable(column(entries).spacing(1.0).padding(Padding {
        top: 2.0,
        right: 4.0,
        bottom: 8.0,
        left: 4.0,
    }))
    .height(Length::Fill);

    let bg_color = theme.palette().base.bg;
    let border_color = theme.palette().base.border;

    container(column![header, content_scroll])
        .width(Length::Fixed(state.sidebar_width))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
            text_color: None,
        })
        .into()
}

fn render_header<'a>(
    state: &'a FileTreeState,
    theme: AppTheme,
    font: iced::Font,
) -> Element<'a, Message> {
    let text_dim = theme.palette().base.text_dim;
    let surface_raised = theme.palette().base.surface_raised;

    let root_icon: Element<'a, Message> = if let Some(svg_handle) =
        icon_theme::get_icon_handle("folder")
            .or_else(|| icon_theme::get_icon_handle("inode-directory"))
    {
        svg(svg_handle).width(14.0).height(14.0).into()
    } else {
        text("•").size(12).font(font).into()
    };

    let root_label = row![
        root_icon,
        text(&state.root_name)
            .size(typography::CAPTION)
            .font(font)
            .color(theme.palette().base.text),
    ]
    .spacing(spacing::XS)
    .align_y(Alignment::Center);

    let collapse_content: Element<'a, Message> = if let Some(svg_handle) =
        icon_theme::get_icon_handle("collapse-all")
            .or_else(|| icon_theme::get_icon_handle("sidebar-collapse"))
    {
        row![
            svg(svg_handle).width(11.0).height(11.0),
            text("Collapse").size(10).font(font).color(text_dim),
        ]
        .spacing(4.0)
        .align_y(Alignment::Center)
        .into()
    } else {
        text("Collapse").size(10).font(font).color(text_dim).into()
    };

    let collapse_btn = button(collapse_content)
        .on_press(FileTreeMsg::CollapseAll.into())
        .padding([2, 5])
        .style(move |_, status| button::Style {
            background: match status {
                button::Status::Hovered | button::Status::Pressed => Some(surface_raised.into()),
                _ => None,
            },
            text_color: text_dim,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        });

    let rule_color = theme.palette().base.rule;

    container(
        row![root_label, Space::new().width(Length::Fill), collapse_btn].align_y(Alignment::Center),
    )
    .padding(HEADER_PADDING)
    .style(move |_| container::Style {
        border: Border {
            color: rule_color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn collect_visible_nodes<'a>(nodes: &'a [FileTreeNode], result: &mut Vec<&'a FileTreeNode>) {
    for node in nodes {
        result.push(node);
        if node.is_dir
            && node.is_expanded
            && let Some(ref children) = node.children
        {
            collect_visible_nodes(children, result);
        }
    }
}

fn render_node_row<'a>(
    node: &'a FileTreeNode,
    current_file_path: &std::path::Path,
    theme: AppTheme,
    font: iced::Font,
) -> Element<'a, Message> {
    let is_active = !node.is_dir && node.path == current_file_path;
    let indent = (node.depth as f32) * 12.0;
    let arrow_color = theme.palette().sidebar.arrow_text;

    let arrow_element: Element<'a, Message> = if node.is_dir {
        let arrow_char = if node.is_expanded { "▾" } else { "▸" };
        container(text(arrow_char).size(10).font(font).color(arrow_color))
            .width(Length::Fixed(12.0))
            .align_x(Alignment::Center)
            .into()
    } else {
        Space::new().width(Length::Fixed(12.0)).into()
    };

    let icon_element = render_tree_node_icon(node);

    let content = row![
        Space::new().width(Length::Fixed(indent)),
        arrow_element,
        icon_element,
        text(&node.name).size(typography::CAPTION).font(font),
    ]
    .spacing(4.0)
    .align_y(Alignment::Center);

    let on_press: Message = if node.is_dir {
        FileTreeMsg::ToggleFolder(node.path.clone()).into()
    } else {
        FileTreeMsg::SelectFile(node.path.clone()).into()
    };

    button(content)
        .on_press(on_press)
        .padding(ENTRY_PADDING)
        .width(Length::Fill)
        .style(move |iced_theme, status| sidebar_entry_style(iced_theme, status, is_active, theme))
        .into()
}

fn render_tree_node_icon<'a>(node: &FileTreeNode) -> Element<'a, Message> {
    let icon_name = if node.is_dir {
        if node.is_expanded {
            "folder-open"
        } else {
            "folder"
        }
    } else {
        icon_theme::icon_for_entry(&node.name, false)
    };

    if let Some(svg_handle) = icon_theme::get_icon_handle(icon_name).or_else(|| {
        if node.is_dir {
            icon_theme::get_icon_handle("inode-directory")
                .or_else(|| icon_theme::get_icon_handle("folder"))
        } else {
            icon_theme::get_icon_handle("text-x-generic")
                .or_else(|| icon_theme::get_icon_handle("application-x-generic"))
        }
    }) {
        svg(svg_handle).width(14.0).height(14.0).into()
    } else {
        let fallback = if node.is_dir { "•" } else { "·" };
        text(fallback).size(12).into()
    }
}
