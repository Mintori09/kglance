//! # dbus-global-menu
//!
//! A universal, lightweight Linux Global Menu library implementing the `com.canonical.dbusmenu`
//! specification for Rust GUI applications and frameworks (GPUI, Iced, Tauri, Winit, Slint, egui).
//!
//! Features:
//! - Full `com.canonical.dbusmenu` D-Bus interface implementation via `zbus`.
//! - Wayland global menu support for KDE Plasma (`org_kde_kwin_appmenu`).
//! - X11 global menu registration via `com.canonical.AppMenu.Registrar`.
//! - Dynamic menu mutations (labels, enable/disable status, checkboxes, radio groups).
//! - Submenu lazy loading on demand (`AboutToShow`).
//! - Standard FreeDesktop accelerator shortcuts and icons.

pub mod dbusmenu;
pub mod wayland;

use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use zbus::Connection;
use zbus::connection::Builder as ZbusBuilder;
use zbus::zvariant::{self, OwnedValue};

pub use dbusmenu::{
    AboutToShowCallback, ActionCallback, DbusMenuService, Disposition, MenuItem, ToggleState,
    ToggleType,
};
pub use wayland::PlasmaAppMenuBinding;

/// Thread-safe, cloneable handle to dynamically manipulate and mutate an active menu tree.
///
/// Updates performed through `MenuHandle` automatically emit `LayoutUpdated` and
/// `ItemsPropertiesUpdated` D-Bus signals to keep the desktop panel in sync.
#[derive(Clone)]
pub struct MenuHandle {
    root: Arc<RwLock<MenuItem>>,
    revision: Arc<RwLock<u32>>,
    connection: Connection,
    object_path: String,
}

impl MenuHandle {
    /// Creates a new `MenuHandle`.
    pub fn new(
        root: Arc<RwLock<MenuItem>>,
        revision: Arc<RwLock<u32>>,
        connection: Connection,
        object_path: impl Into<String>,
    ) -> Self {
        Self {
            root,
            revision,
            connection,
            object_path: object_path.into(),
        }
    }

    /// Returns a clone of the current menu tree.
    pub fn get_root(&self) -> MenuItem {
        self.root.read().clone()
    }

    /// Replaces the entire menu tree and notifies the desktop panel of the layout change.
    pub fn set_root(&self, new_root: MenuItem) {
        let new_rev = {
            let mut root = self.root.write();
            *root = new_root;
            let mut rev = self.revision.write();
            *rev += 1;
            *rev
        };
        self.emit_layout_updated(new_rev, 0);
    }

    /// Clears all children of the root menu item and notifies the desktop panel.
    pub fn set_empty(&self) {
        let new_rev = {
            let mut root = self.root.write();
            root.children.clear();
            let mut rev = self.revision.write();
            *rev += 1;
            *rev
        };
        self.emit_layout_updated(new_rev, 0);
    }

    /// Updates a specific menu item by its ID using a mutation closure.
    ///
    /// Automatically increments the layout revision and emits `LayoutUpdated`.
    pub fn update_item<F>(&self, id: i32, updater: F) -> bool
    where
        F: FnOnce(&mut MenuItem),
    {
        let (updated, new_rev) = {
            let mut root = self.root.write();
            let mut rev = self.revision.write();
            *rev += 1;
            let current_rev = *rev;

            if let Some(item) = DbusMenuService::find_item_by_id_mut(&mut root, id) {
                updater(item);
                (true, current_rev)
            } else {
                (false, current_rev)
            }
        };

        if updated {
            // Emitting layout updated with parent=0 notifies client of top-level changes (such as new submenus)
            self.emit_layout_updated(new_rev, 0);
        }
        updated
    }

    /// Updates the toggle state (checkbox or radio button) for a menu item.
    ///
    /// Emits both `ItemsPropertiesUpdated` and `LayoutUpdated`.
    pub fn set_toggle_state(&self, id: i32, state: impl Into<ToggleState>) -> bool {
        let st = state.into();
        let (updated, new_rev) = {
            let mut root = self.root.write();
            if let Some(item) = DbusMenuService::find_item_by_id_mut(&mut root, id) {
                if item.toggle_state == st {
                    return false;
                }
                item.toggle_state = st;
                let mut rev = self.revision.write();
                *rev += 1;
                (true, *rev)
            } else {
                (false, 0)
            }
        };

        if updated {
            let mut props = HashMap::new();
            props.insert("toggle-state".to_string(), OwnedValue::from(st as i32));
            self.emit_properties_updated(vec![(id, props)], vec![]);
            self.emit_layout_updated(new_rev, id);
        }
        updated
    }

    /// Enables or disables a menu item by ID.
    ///
    /// Emits both `ItemsPropertiesUpdated` and `LayoutUpdated`.
    pub fn set_enabled(&self, id: i32, enabled: bool) -> bool {
        let (updated, new_rev) = {
            let mut root = self.root.write();
            if let Some(item) = DbusMenuService::find_item_by_id_mut(&mut root, id) {
                if item.enabled == enabled {
                    return false;
                }
                item.enabled = enabled;
                let mut rev = self.revision.write();
                *rev += 1;
                (true, *rev)
            } else {
                (false, 0)
            }
        };

        if updated {
            let mut props = HashMap::new();
            props.insert("enabled".to_string(), OwnedValue::from(enabled));
            self.emit_properties_updated(vec![(id, props)], vec![]);
            self.emit_layout_updated(new_rev, id);
        }
        updated
    }

    /// Dynamically changes the label of a menu item by ID.
    ///
    /// Emits both `ItemsPropertiesUpdated` and `LayoutUpdated`.
    pub fn set_label(&self, id: i32, label: impl AsRef<str>) -> bool {
        let label_ref = label.as_ref();
        let (updated, new_rev, label_str) = {
            let mut root = self.root.write();
            if let Some(item) = DbusMenuService::find_item_by_id_mut(&mut root, id) {
                if item.label == label_ref {
                    return false;
                }
                let s = label_ref.to_string();
                item.label = s.clone();
                let mut rev = self.revision.write();
                *rev += 1;
                (true, *rev, s)
            } else {
                (false, 0, String::new())
            }
        };

        if updated {
            let mut props = HashMap::new();
            if let Ok(val) = OwnedValue::try_from(zvariant::Value::from(label_str)) {
                props.insert("label".to_string(), val);
            }
            self.emit_properties_updated(vec![(id, props)], vec![]);
            self.emit_layout_updated(new_rev, id);
        }
        updated
    }

    /// Returns the current revision of the menu layout.
    pub fn revision(&self) -> u32 {
        *self.revision.read()
    }

    /// Increments the revision counter and emits a `LayoutUpdated(revision, 0)` D-Bus signal.
    pub fn notify_layout_updated(&self) {
        let new_rev = {
            let mut rev = self.revision.write();
            *rev += 1;
            *rev
        };
        self.emit_layout_updated(new_rev, 0);
    }

    /// Emits a `LayoutUpdated(revision, parent_id)` D-Bus signal.
    pub fn emit_layout_updated(&self, revision: u32, parent_id: i32) {
        let conn = self.connection.clone();
        let path = self.object_path.clone();
        let emit_task = async move {
            let _ = conn
                .emit_signal(
                    None::<&str>,
                    path.as_str(),
                    "com.canonical.dbusmenu",
                    "LayoutUpdated",
                    &(revision, parent_id),
                )
                .await;
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(emit_task);
        } else {
            std::thread::spawn(move || {
                zbus::block_on(emit_task);
            });
        }
    }

    /// Emits an `ItemsPropertiesUpdated(updated, removed)` D-Bus signal.
    fn emit_properties_updated(
        &self,
        updated_props: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) {
        let conn = self.connection.clone();
        let path = self.object_path.clone();
        let emit_task = async move {
            let _ = conn
                .emit_signal(
                    None::<&str>,
                    path.as_str(),
                    "com.canonical.dbusmenu",
                    "ItemsPropertiesUpdated",
                    &(updated_props, removed_props),
                )
                .await;
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(emit_task);
        } else {
            std::thread::spawn(move || {
                zbus::block_on(emit_task);
            });
        }
    }
}

/// Builder for constructing and exporting a `GlobalMenu` D-Bus service.
pub struct GlobalMenuBuilder {
    service_name: String,
    object_path: String,
    root: MenuItem,
    action_sender: Option<mpsc::UnboundedSender<String>>,
    about_to_show_callback: Option<AboutToShowCallback>,
}

impl GlobalMenuBuilder {
    /// Initializes a builder with the requested D-Bus well-known service name.
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            object_path: "/MenuBar".to_string(),
            root: MenuItem::new(0, "Root"),
            action_sender: None,
            about_to_show_callback: None,
        }
    }

    /// Sets the D-Bus object path where the menu interface will be exposed (defaults to `/MenuBar`).
    pub fn object_path(mut self, path: impl Into<String>) -> Self {
        self.object_path = path.into();
        self
    }

    /// Sets the root `MenuItem` structure for the menu hierarchy.
    pub fn menu(mut self, root: MenuItem) -> Self {
        self.root = root;
        self
    }

    /// Connects an asynchronous channel sender to receive clicked action identifiers.
    pub fn action_sender(mut self, sender: mpsc::UnboundedSender<String>) -> Self {
        self.action_sender = Some(sender);
        self
    }

    /// Registers a callback invoked when a submenu is about to be displayed (`AboutToShow`).
    ///
    /// If the callback returns `true`, the revision number is incremented.
    pub fn on_about_to_show<F>(mut self, cb: F) -> Self
    where
        F: Fn(i32) -> bool + Send + Sync + 'static,
    {
        self.about_to_show_callback = Some(Arc::new(cb));
        self
    }

    /// Starts the D-Bus service, registers the well-known bus name, and exposes `com.canonical.dbusmenu`.
    pub fn start(self) -> Result<GlobalMenu, Box<dyn std::error::Error + Send + Sync>> {
        let action_sender = self.action_sender;
        let callback: Option<ActionCallback> = action_sender.map(|sender| {
            Arc::new(move |action: String| {
                let _ = sender.send(action);
            }) as ActionCallback
        });

        let root_clone = self.root.clone();
        let dbus_service = DbusMenuService::new(self.root, callback, self.about_to_show_callback);
        let root_arc = dbus_service.get_root_arc();
        let revision_arc = dbus_service.get_revision_arc();

        // Initialize D-Bus session connection and export service
        let connection = zbus::block_on(async {
            let conn = ZbusBuilder::session()?
                .name(self.service_name.clone())?
                .serve_at(self.object_path.clone(), dbus_service)?
                .build()
                .await?;

            let reply = conn.request_name(self.service_name.clone()).await;
            log::info!("Requested name '{}' reply: {:?}", self.service_name, reply);

            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(conn)
        })?;

        log::info!(
            "D-Bus Menu Service exported at {}{} (unique_name: {:?})",
            self.service_name,
            self.object_path,
            connection.unique_name()
        );

        let handle = MenuHandle::new(
            root_arc,
            revision_arc,
            connection.clone(),
            self.object_path.clone(),
        );

        Ok(GlobalMenu {
            service_name: self.service_name,
            object_path: self.object_path,
            _dbus_connection: connection,
            binding: None,
            bound_xid: None,
            template_root: root_clone,
            handle,
        })
    }
}

/// Represents an active Global Menu session bound to a D-Bus connection.
pub struct GlobalMenu {
    service_name: String,
    object_path: String,
    _dbus_connection: zbus::Connection,
    binding: Option<PlasmaAppMenuBinding>,
    bound_xid: Option<u32>,
    template_root: MenuItem,
    handle: MenuHandle,
}

impl GlobalMenu {
    /// Creates a new `GlobalMenuBuilder`.
    pub fn builder(service_name: impl Into<String>) -> GlobalMenuBuilder {
        GlobalMenuBuilder::new(service_name)
    }

    /// Sets or updates the template root menu used when restoring the menu upon window attachment.
    pub fn set_template_root(&mut self, root: MenuItem) {
        self.template_root = root;
    }

    /// Returns a `MenuHandle` for thread-safe mutation of the menu.
    pub fn handle(&self) -> MenuHandle {
        self.handle.clone()
    }

    /// Detaches the currently bound window surface and releases desktop appmenu resources.
    pub fn detach_window(&mut self) {
        self.binding = None;
        if let Some(xid) = self.bound_xid.take() {
            let conn = self._dbus_connection.clone();
            let detach_task = async move {
                let _ = conn
                    .call_method(
                        Some("com.canonical.AppMenu.Registrar"),
                        "/com/canonical/AppMenu/Registrar",
                        Some("com.canonical.AppMenu.Registrar"),
                        "UnregisterWindow",
                        &(xid,),
                    )
                    .await;
            };
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(detach_task);
            } else {
                std::thread::spawn(move || {
                    zbus::block_on(detach_task);
                });
            }
        }
        self.handle.set_empty();
    }

    /// Binds an application window to the desktop global menu bar.
    ///
    /// Automatically detects Wayland (`org_kde_kwin_appmenu`) or X11 (`com.canonical.AppMenu.Registrar`)
    /// based on the provided window handles.
    pub fn attach_window<W>(
        &mut self,
        window: &W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        W: HasDisplayHandle + HasWindowHandle + ?Sized,
    {
        // First detach any previously bound window cleanly (without emptying menu)
        self.binding = None;
        if let Some(xid) = self.bound_xid.take() {
            let conn = self._dbus_connection.clone();
            let detach_task = async move {
                let _ = conn
                    .call_method(
                        Some("com.canonical.AppMenu.Registrar"),
                        "/com/canonical/AppMenu/Registrar",
                        Some("com.canonical.AppMenu.Registrar"),
                        "UnregisterWindow",
                        &(xid,),
                    )
                    .await;
            };
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(detach_task);
            } else {
                std::thread::spawn(move || {
                    zbus::block_on(detach_task);
                });
            }
        }

        // Restore root menu if empty
        if self.handle.get_root().children.is_empty() && !self.template_root.children.is_empty() {
            let mut root = self.handle.root.write();
            *root = self.template_root.clone();
        }

        let raw_display = window
            .display_handle()
            .map_err(|e| format!("Failed to get display handle: {:?}", e))?
            .as_raw();
        let raw_window = window
            .window_handle()
            .map_err(|e| format!("Failed to get window handle: {:?}", e))?
            .as_raw();

        let service_address = self
            ._dbus_connection
            .unique_name()
            .map(|n| n.as_str())
            .unwrap_or(&self.service_name);

        match (raw_display, raw_window) {
            (
                raw_window_handle::RawDisplayHandle::Wayland(_),
                raw_window_handle::RawWindowHandle::Wayland(_),
            ) => {
                let binding = PlasmaAppMenuBinding::bind(
                    raw_display,
                    raw_window,
                    service_address,
                    &self.object_path,
                )?;
                self.binding = Some(binding);
                self.handle.notify_layout_updated();
                Ok(())
            }
            (
                raw_window_handle::RawDisplayHandle::Xlib(_)
                | raw_window_handle::RawDisplayHandle::Xcb(_),
                _,
            ) => {
                let xid: u32 = match raw_window {
                    raw_window_handle::RawWindowHandle::Xlib(handle) => handle.window as u32,
                    raw_window_handle::RawWindowHandle::Xcb(handle) => handle.window.get(),
                    _ => return Err("Unsupported window handle for X11 display".into()),
                };

                let conn = self._dbus_connection.clone();
                let path = self.object_path.clone();

                zbus::block_on(async move {
                    log::info!(
                        "Registering X11 window {} with com.canonical.AppMenu.Registrar",
                        xid
                    );
                    let _ = conn
                        .call_method(
                            Some("com.canonical.AppMenu.Registrar"),
                            "/com/canonical/AppMenu/Registrar",
                            Some("com.canonical.AppMenu.Registrar"),
                            "RegisterWindow",
                            &(xid, zvariant::ObjectPath::try_from(path.as_str())?),
                        )
                        .await;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                })?;
                self.bound_xid = Some(xid);
                self.handle.notify_layout_updated();
                Ok(())
            }
            _ => Err("Unsupported display or window backend".into()),
        }
    }
}
