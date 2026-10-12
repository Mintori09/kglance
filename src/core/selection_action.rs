use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomAction {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub popup: bool,
    #[serde(default)]
    pub toast: bool,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionMenuConfig {
    #[serde(default = "default_selection_menu_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub custom_actions: Vec<CustomAction>,
}

fn default_selection_menu_enabled() -> bool {
    true
}

impl Default for SelectionMenuConfig {
    fn default() -> Self {
        Self {
            enabled: default_selection_menu_enabled(),
            custom_actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum SelectionMenuState {
    #[default]
    Hidden,
    Toolbar {
        pos: iced::Point,
        selected_text: String,
    },
    Running {
        pos: iced::Point,
        selected_text: String,
        action_name: String,
    },
    Result {
        pos: iced::Point,
        action_name: String,
        content: String,
        blocks: Vec<crate::parsers::markdown::Block>,
        is_streaming: bool,
    },
}

pub const TOOLBAR_HEIGHT: f32 = 36.0;
pub const RUNNING_WIDTH: f32 = 200.0;
pub const RUNNING_HEIGHT: f32 = 36.0;
pub const RESULT_WIDTH: f32 = 460.0;
pub const RESULT_HEIGHT: f32 = 340.0;

/// Computes the approximate rendered width of the floating selection toolbar given custom actions.
pub fn estimate_toolbar_width(actions: &[CustomAction]) -> f32 {
    let base_width: f32 = 45.0 + 16.0; // Copy button text + padding
    let mut custom_width: f32 = 0.0;
    for action in actions {
        let text_w = (action.name.chars().count() as f32 * 7.5).max(40.0) + 16.0;
        custom_width += text_w;
    }
    let total_buttons = 1 + actions.len();
    let total_spacing = (total_buttons.saturating_sub(1) as f32) * 4.0;
    let container_padding = 16.0;
    (base_width + custom_width + total_spacing + container_padding).max(80.0)
}

impl SelectionMenuState {
    #[inline]
    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::Hidden)
    }

    #[inline]
    pub fn hide(&mut self) {
        *self = Self::Hidden;
    }

    #[inline]
    pub fn position(&self) -> Option<iced::Point> {
        match self {
            Self::Hidden => None,
            Self::Toolbar { pos, .. } | Self::Running { pos, .. } | Self::Result { pos, .. } => {
                Some(*pos)
            }
        }
    }

    #[inline]
    pub fn selected_text(&self) -> Option<&str> {
        match self {
            Self::Toolbar { selected_text, .. } | Self::Running { selected_text, .. } => {
                Some(selected_text.as_str())
            }
            Self::Hidden | Self::Result { .. } => None,
        }
    }

    pub fn bounds(
        &self,
        win_w: f32,
        win_h: f32,
        custom_actions: &[CustomAction],
    ) -> Option<iced::Rectangle> {
        let pos = self.position()?;
        let (menu_w, menu_h) = match self {
            Self::Toolbar { .. } => (
                estimate_toolbar_width(custom_actions).min((win_w - 24.0).max(220.0)),
                TOOLBAR_HEIGHT,
            ),
            Self::Running { .. } => (RUNNING_WIDTH, RUNNING_HEIGHT),
            Self::Result { .. } => (RESULT_WIDTH, RESULT_HEIGHT),
            Self::Hidden => return None,
        };
        let clamped_x = (pos.x - 24.0).clamp(12.0, (win_w - menu_w - 12.0).max(12.0));
        let clamped_y = if pos.y - menu_h - 12.0 >= 44.0 {
            pos.y - menu_h - 12.0
        } else {
            (pos.y + 16.0).clamp(44.0, (win_h - menu_h - 12.0).max(44.0))
        };
        Some(iced::Rectangle {
            x: (clamped_x - 4.0).max(0.0),
            y: (clamped_y - 4.0).max(0.0),
            width: menu_w + 8.0,
            height: menu_h + 8.0,
        })
    }

    #[inline]
    pub fn is_cursor_inside(
        &self,
        cursor: Option<iced::Point>,
        win_w: f32,
        win_h: f32,
        custom_actions: &[CustomAction],
    ) -> bool {
        if let Some(cursor) = cursor
            && let Some(bounds) = self.bounds(win_w, win_h, custom_actions)
        {
            bounds.contains(cursor)
        } else {
            false
        }
    }

    /// Returns `true` when the menu must not be dismissed by implicit selection/drag events.
    ///
    /// This covers:
    /// - `Running`: a background task is in flight.
    /// - `Result`: a popup card is visible (streaming or fully loaded). Once results are shown,
    ///   only an explicit `Dismiss` action may close the popup — never a mouse drag or clear event.
    ///   This prevents instant-result actions from flickering due to drag events arriving after
    ///   `ActionResult` has already set `is_streaming: false`.
    #[inline]
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Running { .. } | Self::Result { .. })
    }
}

pub async fn execute_custom_action(command: String, text: String) -> Result<String, String> {
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .env("KGLANCE_TEXT", &text)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(text.as_bytes()).await;
        let _ = stdin.flush().await;
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to execute command: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if stderr.trim().is_empty() {
            let code = output.status.code().unwrap_or(-1);
            Err(format!("Process exited with status {code}"))
        } else {
            Err(stderr.trim().to_string())
        }
    }
}

#[cfg(test)]
#[path = "selection_action_tests.rs"]
mod tests;
