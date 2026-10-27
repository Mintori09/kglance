use iced::widget::{column, container, row, text};
use iced::{Color, Element, Length};

use crate::app::Message;
use crate::core::FolderRowState;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::{radius, spacing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Code,
    Docs,
    Data,
    Media,
    Folder,
    Other,
}

impl FileCategory {
    pub fn from_kind_or_name(kind: &str, is_dir: bool) -> Self {
        if is_dir {
            return Self::Folder;
        }

        let k = kind.to_ascii_lowercase();
        if k.contains("config")
            || k.contains("toml")
            || k.contains("json")
            || k.contains("yaml")
            || k.contains("lock")
            || k.contains("git")
        {
            Self::Data
        } else if k.contains("document")
            || k.contains("markdown")
            || k.contains("pdf")
            || k.contains("license")
            || k.contains("text")
        {
            Self::Docs
        } else if k.contains("script")
            || k.contains("rust")
            || k.contains("python")
            || k.contains("c ")
            || k.contains("cpp")
            || k.contains("javascript")
            || k.contains("typescript")
            || k.contains("html")
        {
            Self::Code
        } else if k.contains("image") || k.contains("audio") || k.contains("video") {
            Self::Media
        } else {
            Self::Other
        }
    }

    pub fn color(&self, is_dark: bool) -> Color {
        match self {
            Self::Folder => {
                if is_dark {
                    Color::from_rgb(0.35, 0.65, 0.95)
                } else {
                    Color::from_rgb(0.18, 0.50, 0.85)
                }
            }
            Self::Code => {
                if is_dark {
                    Color::from_rgb(0.30, 0.85, 0.55)
                } else {
                    Color::from_rgb(0.15, 0.68, 0.38)
                }
            }
            Self::Docs => {
                if is_dark {
                    Color::from_rgb(0.95, 0.75, 0.30)
                } else {
                    Color::from_rgb(0.85, 0.60, 0.15)
                }
            }
            Self::Data => {
                if is_dark {
                    Color::from_rgb(0.85, 0.45, 0.85)
                } else {
                    Color::from_rgb(0.70, 0.30, 0.70)
                }
            }
            Self::Media => {
                if is_dark {
                    Color::from_rgb(0.95, 0.45, 0.45)
                } else {
                    Color::from_rgb(0.85, 0.30, 0.30)
                }
            }
            Self::Other => {
                if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.35)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.35)
                }
            }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Folder => "Folders",
            Self::Code => "Code",
            Self::Docs => "Docs",
            Self::Data => "Config/Data",
            Self::Media => "Media",
            Self::Other => "Other",
        }
    }
}

pub fn create_distribution_bar<'a>(
    rows: &'a [FolderRowState],
    theme: AppTheme,
) -> Option<Element<'a, Message>> {
    if rows.is_empty() {
        return None;
    }

    let is_dark = theme.is_dark();
    let mut code_bytes = 0u64;
    let mut docs_bytes = 0u64;
    let mut data_bytes = 0u64;
    let mut media_bytes = 0u64;
    let mut other_bytes = 0u64;

    for row in rows {
        if row.is_dir {
            continue;
        }
        let cat = FileCategory::from_kind_or_name(&row.kind, false);
        match cat {
            FileCategory::Code => code_bytes += row.raw_size,
            FileCategory::Docs => docs_bytes += row.raw_size,
            FileCategory::Data => data_bytes += row.raw_size,
            FileCategory::Media => media_bytes += row.raw_size,
            _ => other_bytes += row.raw_size,
        }
    }

    let non_dir_size = code_bytes + docs_bytes + data_bytes + media_bytes + other_bytes;
    if non_dir_size == 0 {
        return None;
    }

    let categories = [
        (FileCategory::Code, code_bytes),
        (FileCategory::Docs, docs_bytes),
        (FileCategory::Data, data_bytes),
        (FileCategory::Media, media_bytes),
        (FileCategory::Other, other_bytes),
    ];

    let mut segments = row![].spacing(2.0).width(Length::Fill).height(6.0);
    let mut legend_items = row![].spacing(spacing::M);

    for (cat, bytes) in categories {
        if bytes == 0 {
            continue;
        }
        let portion = ((bytes as f64 / non_dir_size as f64) * 1000.0).max(1.0) as u16;
        let cat_color = cat.color(is_dark);

        let segment = container(iced::widget::Space::new())
            .width(Length::FillPortion(portion))
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(cat_color.into()),
                border: iced::Border {
                    radius: radius::XS.into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            });

        segments = segments.push(segment);

        let pct = (bytes as f64 / non_dir_size as f64) * 100.0;
        let dot = container(iced::widget::Space::new())
            .width(6.0)
            .height(6.0)
            .style(move |_| container::Style {
                background: Some(cat_color.into()),
                border: iced::Border {
                    radius: radius::FULL.into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            });

        let label = text(format!("{} {:.0}%", cat.label(), pct))
            .size(10.5)
            .color(if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.65)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.65)
            });

        let item = row![dot, label]
            .spacing(spacing::XXS)
            .align_y(iced::Alignment::Center);

        legend_items = legend_items.push(item);
    }

    Some(
        column![segments, legend_items]
            .spacing(spacing::XS)
            .width(Length::Fill)
            .into(),
    )
}
