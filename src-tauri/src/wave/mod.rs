//! Wave Shell additions on top of iBC Shell, kept in their own module so upstream merges stay clean.

pub(crate) mod app_menu;
pub(crate) mod coalesce;
pub(crate) mod net;
pub(crate) mod window_buttons;

/// The menu bar's app items (see `app_menu`); the tray's own items are ignored there.
#[must_use]
pub fn with_app_menu(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    #[cfg(desktop)]
    {
        builder.on_menu_event(|app, e| app_menu::on_event(app, e.id().as_ref()))
    }
    #[cfg(not(desktop))]
    {
        builder
    }
}

/// Self-update and the relaunch after it (desktop only).
///
/// The updater reads `latest.json` from our GitHub releases (see `plugins.updater` in `tauri.conf.json`) and
/// installs only packages signed with our key; the UI drives it through `@tauri-apps/plugin-updater`.
#[must_use]
pub fn with_updater<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    #[cfg(desktop)]
    {
        builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init())
    }
    #[cfg(not(desktop))]
    {
        builder
    }
}
