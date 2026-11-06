use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

/// Internal Wayland event handling state.
pub struct WaylandMenuState {
    pub appmenu_manager: Option<OrgKdeKwinAppmenuManager>,
}

impl Dispatch<WlRegistry, GlobalListContents> for WaylandMenuState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<OrgKdeKwinAppmenuManager, ()> for WaylandMenuState {
    fn event(
        _state: &mut Self,
        _proxy: &OrgKdeKwinAppmenuManager,
        _event: <OrgKdeKwinAppmenuManager as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<OrgKdeKwinAppmenu, ()> for WaylandMenuState {
    fn event(
        _state: &mut Self,
        _proxy: &OrgKdeKwinAppmenu,
        _event: <OrgKdeKwinAppmenu as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

/// Wayland surface binding with KDE Plasma's `org_kde_kwin_appmenu`.
pub struct PlasmaAppMenuBinding {
    pub appmenu: OrgKdeKwinAppmenu,
    pub connection: Connection,
}

impl PlasmaAppMenuBinding {
    /// Registers a Wayland window surface with KDE KWin's Global Menu manager.
    pub fn bind(
        raw_display: RawDisplayHandle,
        raw_window: RawWindowHandle,
        dbus_service_name: &str,
        dbus_object_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let RawDisplayHandle::Wayland(display_handle) = raw_display else {
            return Err("Display handle is not Wayland".into());
        };

        let RawWindowHandle::Wayland(window_handle) = raw_window else {
            return Err("Window handle is not Wayland".into());
        };

        // 1. Create client backend from foreign display pointer
        let backend = unsafe {
            wayland_backend::sys::client::Backend::from_foreign_display(
                display_handle.display.as_ptr() as *mut _,
            )
        };
        let connection = Connection::from_backend(backend);

        // 2. Initialize registry queue to discover KWin AppMenu Manager
        let (globals, mut event_queue) = registry_queue_init::<WaylandMenuState>(&connection)?;
        let qh = event_queue.handle();

        let mut state = WaylandMenuState {
            appmenu_manager: None,
        };

        // Locate and bind org_kde_kwin_appmenu_manager
        let manager: OrgKdeKwinAppmenuManager = globals
            .bind(&qh, 1..=2, ())
            .map_err(|e| format!("Failed to bind org_kde_kwin_appmenu_manager: {:?}", e))?;

        // 3. Reconstruct WlSurface proxy from raw window pointer
        let surface_ptr = window_handle.surface.as_ptr();
        let surface_id = unsafe {
            wayland_backend::sys::client::ObjectId::from_ptr(
                WlSurface::interface(),
                surface_ptr as *mut _,
            )
        }
        .map_err(|e| format!("Failed to create ObjectId from surface pointer: {:?}", e))?;

        let surface: WlSurface = WlSurface::from_id(&connection, surface_id)
            .map_err(|e| format!("Failed to recreate WlSurface proxy: {:?}", e))?;

        // 4. Create org_kde_kwin_appmenu for this surface and register D-Bus address
        let appmenu = manager.create(&surface, &qh, ());
        appmenu.set_address(dbus_service_name.to_string(), dbus_object_path.to_string());

        // Perform roundtrip to ensure compositor received the request
        event_queue.roundtrip(&mut state)?;

        log::info!(
            "Successfully bound Wayland surface to KDE AppMenu: service={}, path={}",
            dbus_service_name,
            dbus_object_path
        );

        Ok(Self {
            appmenu,
            connection,
        })
    }
}

impl Drop for PlasmaAppMenuBinding {
    fn drop(&mut self) {
        if self.appmenu.is_alive() {
            self.appmenu.release();
            let _ = self.connection.flush();
        }
    }
}
