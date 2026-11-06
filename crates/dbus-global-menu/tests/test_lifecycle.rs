use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use zbus::export::ordered_stream::OrderedStreamExt;

struct DummyWindow {
    display: RawDisplayHandle,
    window: RawWindowHandle,
}

impl HasDisplayHandle for DummyWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { Ok(DisplayHandle::borrow_raw(self.display)) }
    }
}

impl HasWindowHandle for DummyWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { Ok(WindowHandle::borrow_raw(self.window)) }
    }
}

struct AppState {
    compositor: Option<WlCompositor>,
}

impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "wl_compositor"
        {
            state.compositor = Some(proxy.bind(name, version.min(4), qh, ()));
        }
    }
}

impl Dispatch<WlCompositor, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: <WlCompositor as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: <WlSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

/// Helper to set up Wayland connection and DummyWindow
fn setup_wayland_window() -> Option<(
    Connection,
    wayland_client::EventQueue<AppState>,
    DummyWindow,
    WlSurface,
)> {
    let conn = Connection::connect_to_env().ok()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    let mut state = AppState { compositor: None };
    event_queue.roundtrip(&mut state).ok()?;
    let compositor = state.compositor.clone()?;

    let raw_display = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
        NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
    ));

    let surface = compositor.create_surface(&qh, ());
    event_queue.roundtrip(&mut state).ok()?;

    let win = DummyWindow {
        display: raw_display,
        window: RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
        )),
    };

    Some((conn, event_queue, win, surface))
}

#[tokio::test(flavor = "multi_thread")]
async fn test_solution_attach_emits_layout_updated_with_incremented_revision() {
    let Some((_conn, _queue, win, _surface)) = setup_wayland_window() else {
        println!("Skipping: Wayland not available");
        return;
    };

    const SERVICE_NAME: &str = "org.mintori.Kglance.SolutionSignalTest";
    let object_path = "/MenuBarSolution1";

    let mut menu = dbus_global_menu::GlobalMenu::builder(SERVICE_NAME)
        .object_path(object_path)
        .menu(dbus_global_menu::MenuItem::new(0, "Root"))
        .start()
        .expect("Failed to start menu");

    let initial_rev = menu.handle().revision();
    assert_eq!(initial_rev, 1);

    // Monitor LayoutUpdated signal specifically on object_path
    let session_conn = zbus::Connection::session().await.unwrap();
    let signal_count = Arc::new(AtomicUsize::new(0));
    let last_revision = Arc::new(AtomicU32::new(0));

    let signal_count_clone = signal_count.clone();
    let last_revision_clone = last_revision.clone();
    let target_path = object_path.to_string();

    let rule = format!(
        "type='signal',interface='com.canonical.dbusmenu',member='LayoutUpdated',path='{target_path}'"
    );
    session_conn
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "AddMatch",
            &(rule,),
        )
        .await
        .unwrap();

    let mut msg_stream = zbus::MessageStream::from(&session_conn);
    tokio::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            if let Some(path) = msg.header().path()
                && path.as_str() == target_path
                && let Ok((rev, parent_id)) = msg.body().deserialize::<(u32, i32)>()
            {
                assert_eq!(parent_id, 0);
                last_revision_clone.store(rev, Ordering::SeqCst);
                signal_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Call attach_window
    let res = menu.attach_window(&win);
    assert!(res.is_ok(), "attach_window should succeed");

    // Wait for D-Bus signal to arrive
    std::thread::sleep(std::time::Duration::from_millis(150));

    let count = signal_count.load(Ordering::SeqCst);
    let rev = last_revision.load(Ordering::SeqCst);
    println!("[VERIFICATION] Signals received: {count}, layout revision: {rev}");

    assert_eq!(
        count, 1,
        "attach_window must emit exactly 1 LayoutUpdated signal!"
    );
    assert_eq!(
        rev, 2,
        "Revision must be incremented from 1 to 2 upon window attachment!"
    );
    assert_eq!(menu.handle().revision(), 2);
}

#[test]
fn test_solution_detach_window_drops_binding_and_releases_resources() {
    let Some((_conn, mut queue, win, _surface)) = setup_wayland_window() else {
        println!("Skipping: Wayland not available");
        return;
    };

    const SERVICE_NAME: &str = "org.mintori.Kglance.SolutionDetachTest";
    let mut menu = dbus_global_menu::GlobalMenu::builder(SERVICE_NAME)
        .object_path("/MenuBarSolution2")
        .menu(dbus_global_menu::MenuItem::new(0, "Root"))
        .start()
        .expect("Failed to start menu");

    // Attach
    let attach_res = menu.attach_window(&win);
    assert!(attach_res.is_ok());

    let mut state = AppState { compositor: None };
    let rt1 = queue.roundtrip(&mut state);
    assert!(rt1.is_ok());

    // Detach window
    println!("[TEST] Calling menu.detach_window()...");
    menu.detach_window();

    // Roundtrip to compositor to verify release() was received and handled cleanly
    let rt2 = queue.roundtrip(&mut state);
    println!("[VERIFICATION] Roundtrip after detach_window(): {:?}", rt2);
    assert!(
        rt2.is_ok(),
        "Compositor roundtrip must succeed after release()"
    );
}

#[test]
fn test_solution_sequential_lifecycle_simulation() {
    let Some((_conn, mut queue, win, surface)) = setup_wayland_window() else {
        println!("Skipping: Wayland not available");
        return;
    };

    const SERVICE_NAME: &str = "org.mintori.Kglance.SolutionLifecycleTest";
    let mut menu = dbus_global_menu::GlobalMenu::builder(SERVICE_NAME)
        .object_path("/MenuBarSolution3")
        .menu(dbus_global_menu::MenuItem::new(0, "Root"))
        .start()
        .expect("Failed to start menu");

    let mut state = AppState { compositor: None };

    for cycle in 1..=3 {
        println!("[LIFECYCLE CYCLE {cycle}] Attaching window...");
        let attach_res = menu.attach_window(&win);
        assert!(attach_res.is_ok());
        assert_eq!(menu.handle().revision(), 1 + cycle as u32);

        let rt_attach = queue.roundtrip(&mut state);
        assert!(rt_attach.is_ok());

        println!("[LIFECYCLE CYCLE {cycle}] Detaching window (closing window)...");
        menu.detach_window();

        let rt_detach = queue.roundtrip(&mut state);
        assert!(rt_detach.is_ok());
    }

    surface.destroy();
}
