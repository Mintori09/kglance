use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use zbus::interface;
use zbus::zvariant::{OwnedValue, Value};

/// State of a Checkbox or Radio menu item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToggleState {
    Off = 0,
    On = 1,
    Indeterminate = 2,
}

impl From<bool> for ToggleState {
    fn from(val: bool) -> Self {
        if val {
            ToggleState::On
        } else {
            ToggleState::Off
        }
    }
}

/// Type of toggle control for an item (Checkbox or Radio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToggleType {
    Checkmark,
    Radio,
}

impl ToggleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToggleType::Checkmark => "checkmark",
            ToggleType::Radio => "radio",
        }
    }
}

/// Visual disposition of a menu item as defined in the DBusMenu specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Normal,
    Informative,
    Warning,
    Alert,
}

impl Disposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Disposition::Normal => "normal",
            Disposition::Informative => "informative",
            Disposition::Warning => "warning",
            Disposition::Alert => "alert",
        }
    }
}

/// Represents a single menu item in the `com.canonical.dbusmenu` hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub icon_name: Option<String>,
    pub icon_data: Option<Vec<u8>>,
    /// Keyboard shortcut formatted as an array of arrays of strings (`aas`).
    pub shortcut: Option<Vec<Vec<String>>>,
    pub toggle_type: Option<ToggleType>,
    pub toggle_state: ToggleState,
    pub disposition: Disposition,
    pub children: Vec<MenuItem>,
    pub action_id: Option<String>,
}

impl MenuItem {
    /// Creates a new `MenuItem` with the given ID and text label.
    pub fn new(id: i32, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            enabled: true,
            visible: true,
            icon_name: None,
            icon_data: None,
            shortcut: None,
            toggle_type: None,
            toggle_state: ToggleState::Off,
            disposition: Disposition::Normal,
            children: Vec::new(),
            action_id: None,
        }
    }

    /// Associates an action identifier string triggered upon activation.
    pub fn with_action(mut self, action_id: impl Into<String>) -> Self {
        self.action_id = Some(action_id.into());
        self
    }

    /// Sets an accelerator shortcut using individual key components, e.g. `vec!["Control", "S"]`.
    pub fn with_shortcut(mut self, keys: Vec<String>) -> Self {
        self.shortcut = Some(vec![keys]);
        self
    }

    /// Sets an accelerator shortcut from a string representation such as `"Ctrl+Shift+Z"` or `"Control+O"`.
    pub fn with_shortcut_str(mut self, s: &str) -> Self {
        let parts: Vec<String> = s
            .split('+')
            .map(|part| {
                let p = part.trim();
                match p.to_lowercase().as_str() {
                    "ctrl" | "control" => "Control".to_string(),
                    "alt" => "Alt".to_string(),
                    "shift" => "Shift".to_string(),
                    "super" | "win" | "cmd" | "meta" => "Super".to_string(),
                    _ => p.to_string(),
                }
            })
            .collect();
        self.shortcut = Some(vec![parts]);
        self
    }

    /// Configures the item as a checkbox with an initial state.
    pub fn with_checkbox(mut self, state: impl Into<ToggleState>) -> Self {
        self.toggle_type = Some(ToggleType::Checkmark);
        self.toggle_state = state.into();
        self
    }

    /// Configures the item as a radio button with an initial state.
    pub fn with_radio(mut self, state: impl Into<ToggleState>) -> Self {
        self.toggle_type = Some(ToggleType::Radio);
        self.toggle_state = state.into();
        self
    }

    /// Sets a themed FreeDesktop icon name (e.g. `"document-save"`, `"edit-undo"`).
    pub fn with_icon(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(icon_name.into());
        self
    }

    /// Sets the visual disposition (e.g. `Disposition::Alert`, `Disposition::Informative`).
    pub fn with_disposition(mut self, disposition: Disposition) -> Self {
        self.disposition = disposition;
        self
    }

    /// Configures whether the item is active and interactive.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Configures whether the item is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Appends child items to construct a submenu.
    pub fn with_children(mut self, children: Vec<MenuItem>) -> Self {
        self.children = children;
        self
    }

    /// Creates a visual separator item (`type = "separator"`).
    pub fn separator(id: i32) -> Self {
        let mut item = Self::new(id, "");
        item.action_id = Some("separator".to_string());
        item
    }

    /// Exports item properties conforming to the `com.canonical.dbusmenu` specification.
    pub fn to_properties(&self, filter: &[String]) -> HashMap<String, Value<'static>> {
        let mut map = HashMap::new();
        let include_all = filter.is_empty();

        let should_include = |k: &str| include_all || filter.iter().any(|f| f == k);

        if should_include("label") && !self.label.is_empty() {
            map.insert("label".to_string(), Value::from(self.label.clone()));
        }

        if should_include("enabled") {
            map.insert("enabled".to_string(), Value::from(self.enabled));
        }

        if should_include("visible") {
            map.insert("visible".to_string(), Value::from(self.visible));
        }

        if let Some(ref icon) = self.icon_name
            && should_include("icon-name")
        {
            map.insert("icon-name".to_string(), Value::from(icon.clone()));
        }

        if let Some(ref icon_bytes) = self.icon_data
            && should_include("icon-data")
        {
            map.insert("icon-data".to_string(), Value::from(icon_bytes.clone()));
        }

        if self.action_id.as_deref() == Some("separator") && should_include("type") {
            map.insert("type".to_string(), Value::from("separator"));
        }

        if !self.children.is_empty() && should_include("children-display") {
            map.insert("children-display".to_string(), Value::from("submenu"));
        }

        if let Some(toggle_type) = self.toggle_type {
            if should_include("toggle-type") {
                map.insert("toggle-type".to_string(), Value::from(toggle_type.as_str()));
            }
            if should_include("toggle-state") {
                map.insert(
                    "toggle-state".to_string(),
                    Value::from(self.toggle_state as i32),
                );
            }
        }

        if self.disposition != Disposition::Normal && should_include("disposition") {
            map.insert(
                "disposition".to_string(),
                Value::from(self.disposition.as_str()),
            );
        }

        if let Some(ref shortcut) = self.shortcut
            && should_include("shortcut")
        {
            // Signature: aas (Array of arrays of strings)
            let sc_value = Value::from(shortcut.clone());
            map.insert("shortcut".to_string(), sc_value);
        }

        map
    }
}

pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;
pub type AboutToShowCallback = Arc<dyn Fn(i32) -> bool + Send + Sync + 'static>;

/// Implementation of the `com.canonical.dbusmenu` D-Bus interface.
pub struct DbusMenuService {
    root: Arc<RwLock<MenuItem>>,
    revision: Arc<RwLock<u32>>,
    on_action: Option<ActionCallback>,
    on_about_to_show: Option<AboutToShowCallback>,
}

impl DbusMenuService {
    /// Creates a new `DbusMenuService` with the specified root menu item and optional callbacks.
    pub fn new(
        root: MenuItem,
        on_action: Option<ActionCallback>,
        on_about_to_show: Option<AboutToShowCallback>,
    ) -> Self {
        Self {
            root: Arc::new(RwLock::new(root)),
            revision: Arc::new(RwLock::new(1)),
            on_action,
            on_about_to_show,
        }
    }

    /// Returns a reference to the shared root `MenuItem`.
    pub fn get_root_arc(&self) -> Arc<RwLock<MenuItem>> {
        self.root.clone()
    }

    /// Returns a reference to the shared layout revision number.
    pub fn get_revision_arc(&self) -> Arc<RwLock<u32>> {
        self.revision.clone()
    }

    /// Recursively searches for an item by ID within the menu hierarchy.
    pub fn find_item_by_id(item: &MenuItem, id: i32) -> Option<&MenuItem> {
        if item.id == id {
            return Some(item);
        }
        for child in &item.children {
            if let Some(found) = Self::find_item_by_id(child, id) {
                return Some(found);
            }
        }
        None
    }

    /// Recursively searches for a mutable reference to an item by ID within the menu hierarchy.
    pub fn find_item_by_id_mut(item: &mut MenuItem, id: i32) -> Option<&mut MenuItem> {
        if item.id == id {
            return Some(item);
        }
        for child in &mut item.children {
            if let Some(found) = Self::find_item_by_id_mut(child, id) {
                return Some(found);
            }
        }
        None
    }

    fn build_layout_value(
        item: &MenuItem,
        recursion_depth: i32,
        property_names: &[String],
    ) -> Value<'static> {
        let props = item.to_properties(property_names);
        let mut props_owned: HashMap<String, OwnedValue> = HashMap::new();
        for (k, v) in props {
            if let Ok(owned_val) = OwnedValue::try_from(v) {
                props_owned.insert(k, owned_val);
            }
        }

        let mut children_values = Vec::new();
        if recursion_depth != 0 {
            let next_depth = if recursion_depth > 0 {
                recursion_depth - 1
            } else {
                -1
            };
            for child in &item.children {
                children_values.push(Self::build_layout_value(child, next_depth, property_names));
            }
        }

        // Layout format: (ia{sv}av)
        Value::from((item.id, props_owned, children_values))
    }
}

#[interface(name = "com.canonical.dbusmenu")]
impl DbusMenuService {
    /// Provides the layout of the menu hierarchy rooted at `parent_id`.
    pub async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
        let root = self.root.read();
        let rev = *self.revision.read();

        let item = if parent_id == 0 || root.id == parent_id {
            &*root
        } else {
            match Self::find_item_by_id(&root, parent_id) {
                Some(it) => it,
                None => &*root,
            }
        };

        let val = Self::build_layout_value(item, recursion_depth, &property_names);
        if let Value::Structure(fields) = val {
            let mut iter = fields.into_fields().into_iter();
            let id = i32::try_from(iter.next().unwrap_or(Value::from(0i32))).unwrap_or(0);
            let props_val = iter
                .next()
                .unwrap_or(Value::from(HashMap::<String, OwnedValue>::new()));
            let children_val = iter.next().unwrap_or(Value::from(Vec::<OwnedValue>::new()));

            let props: HashMap<String, OwnedValue> =
                HashMap::try_from(props_val).unwrap_or_default();
            let children: Vec<OwnedValue> = Vec::try_from(children_val).unwrap_or_default();

            return (rev, (id, props, children));
        }

        (rev, (item.id, HashMap::new(), Vec::new()))
    }

    /// Retrieves properties for a list of menu item IDs in a single call.
    pub async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let root = self.root.read();
        let mut result = Vec::new();

        for id in ids {
            if let Some(item) = Self::find_item_by_id(&root, id) {
                let props = item.to_properties(&property_names);
                let mut owned: HashMap<String, OwnedValue> = HashMap::new();
                for (k, v) in props {
                    if let Ok(owned_val) = OwnedValue::try_from(v) {
                        owned.insert(k, owned_val);
                    }
                }
                result.push((id, owned));
            }
        }

        result
    }

    /// Retrieves a single property value for a specific menu item.
    pub async fn get_property(&self, id: i32, name: String) -> OwnedValue {
        let root = self.root.read();
        if let Some(item) = Self::find_item_by_id(&root, id) {
            let props = item.to_properties(std::slice::from_ref(&name));
            if let Some(val) = props.get(&name)
                && let Ok(owned) = OwnedValue::try_from(val)
            {
                return owned;
            }
        }
        OwnedValue::try_from(Value::from("")).unwrap()
    }

    /// Handles events sent from the menu client (such as `"clicked"`).
    pub async fn event(&self, id: i32, event_id: String, _data: Value<'_>, _timestamp: u32) {
        log::debug!("dbusmenu Event: id={}, event_id={}", id, event_id);
        if event_id == "clicked" {
            let root = self.root.read();
            if let Some(item) = Self::find_item_by_id(&root, id)
                && let Some(ref action) = item.action_id
            {
                log::info!("dbusmenu item clicked, triggering action: {}", action);
                if let Some(ref cb) = self.on_action {
                    cb(action.clone());
                }
            }
        }
    }

    /// Handles a batch of menu events.
    pub async fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        let failed = Vec::new();
        for (id, event_id, data, ts) in events {
            self.event(id, event_id, Value::from(data), ts).await;
        }
        failed
    }

    /// Invoked by the client before displaying a submenu to allow dynamic on-demand population.
    pub async fn about_to_show(&self, id: i32) -> bool {
        log::debug!("dbusmenu AboutToShow: id={}", id);
        if let Some(ref cb) = self.on_about_to_show {
            let need_update = cb(id);
            if need_update {
                let mut rev = self.revision.write();
                *rev += 1;
            }
            return need_update;
        }
        false
    }

    /// Invoked by the client before displaying multiple submenus in batch.
    pub async fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let mut updates_needed = Vec::new();
        if let Some(ref cb) = self.on_about_to_show {
            let mut any_update = false;
            for id in &ids {
                if cb(*id) {
                    updates_needed.push(*id);
                    any_update = true;
                }
            }
            if any_update {
                let mut rev = self.revision.write();
                *rev += 1;
            }
        }
        (updates_needed, Vec::new())
    }

    // --- Properties ---

    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }

    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".to_string()
    }

    #[zbus(property)]
    fn status(&self) -> String {
        "normal".to_string()
    }

    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }
}
