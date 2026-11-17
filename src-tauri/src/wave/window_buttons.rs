//! Wave: the macOS window buttons (close / minimise / zoom) follow the left menu, like in Arc. The menu is
//! hidden — the buttons hide too and the top bar starts at the window edge; the menu is open or slides out —
//! they are back, in its header. Tauri has no call for it; the calls below are safe ones: the window is found
//! among `NSApp.windows` by comparing addresses, nothing is dereferenced by hand.

/// Shows or hides the window buttons of the calling window (macOS; elsewhere nothing). Passive.
#[tauri::command]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the window by value.
pub fn wave_window_buttons(
    window: tauri::WebviewWindow,
    visible: bool,
) -> crate::api::ApiResult<()> {
    #[cfg(target_os = "macos")]
    {
        let target = window
            .ns_window()
            .map_err(|_| crate::api::ApiError::Internal("no native window".into()))?
            as usize;
        window
            .run_on_main_thread(move || set_visible(target, visible))
            .map_err(|_| crate::api::ApiError::Internal("main thread unavailable".into()))?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (window, visible);
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_visible(target: usize, visible: bool) {
    use objc2::rc::Retained;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWindowButton};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let Some(win) = windows
        .iter()
        .find(|w| Retained::as_ptr(w) as usize == target)
    else {
        return;
    };
    for b in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        if let Some(button) = win.standardWindowButton(b) {
            button.setHidden(!visible);
        }
    }
}
