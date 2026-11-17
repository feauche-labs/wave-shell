//! Wave: the macOS menu bar («Wave Shell», «Файл», «Правка», «Вид», «Окно», «Справка») in the app's language.
//!
//! Without it the system showed Tauri's default menu: English only, and nothing of the app in it. The UI knows
//! the language, so it sends the labels (`app_menu_set`); the backend builds a native menu. Standard items
//! (Copy, Hide, Quit…) stay native; the app's own items come back to the focused window as `AppMenuPicked`.
//! Windows and Linux draw no menu bar for the app (the window is the app's own), so there it does nothing.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Every text of the menu, already translated by the UI.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct AppMenuLabels {
    /// «О Wave Shell».
    pub about: String,
    /// «Проверить обновления…».
    pub check_update: String,
    /// «Настройки…».
    pub settings: String,
    /// «Ключи».
    pub keys: String,
    /// «Службы».
    pub services: String,
    /// «Скрыть Wave Shell».
    pub hide: String,
    /// «Скрыть остальные».
    pub hide_others: String,
    /// «Показать все».
    pub show_all: String,
    /// «Завершить Wave Shell».
    pub quit: String,
    /// «Файл».
    pub file: String,
    /// «Новая вкладка».
    pub new_tab: String,
    /// «Локальный терминал».
    pub local_terminal: String,
    /// «Новый сервер…».
    pub new_host: String,
    /// «Закрыть вкладку».
    pub close_tab: String,
    /// «Правка».
    pub edit: String,
    /// «Отменить».
    pub undo: String,
    /// «Повторить».
    pub redo: String,
    /// «Вырезать».
    pub cut: String,
    /// «Копировать».
    pub copy: String,
    /// «Вставить».
    pub paste: String,
    /// «Выбрать всё».
    pub select_all: String,
    /// «Вид».
    pub view: String,
    /// «Показать или скрыть левое меню».
    pub toggle_sidebar: String,
    /// «Крупнее».
    pub scale_in: String,
    /// «Мельче».
    pub scale_out: String,
    /// «Обычный размер».
    pub scale_reset: String,
    /// «Полноэкранный режим».
    pub fullscreen: String,
    /// «Окно».
    pub window: String,
    /// «Свернуть».
    pub minimize: String,
    /// «Увеличить».
    pub zoom: String,
    /// «Справка».
    pub help: String,
    /// «Горячие клавиши».
    pub hotkeys: String,
    /// «Страница проекта на GitHub».
    pub project_page: String,
}

/// An app item of the menu bar; the UI does what the same button or key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AppMenuAction {
    /// Open the update sheet.
    CheckUpdate,
    /// Open the settings (⌘,).
    Settings,
    /// Open the «Keys» page.
    Keys,
    /// New tab (⌘T).
    NewTab,
    /// To the local terminal.
    LocalTerminal,
    /// New server form.
    NewHost,
    /// Close the tab (⌘W); with no tab — the window, as before.
    CloseTab,
    /// Show / hide the left menu (⌘B).
    ToggleSidebar,
    /// Interface scale up (⌘⇧+).
    ScaleIn,
    /// Interface scale down (⌘⇧−).
    ScaleOut,
    /// Interface scale back to 100 % (⌘⇧0).
    ScaleReset,
    /// The keyboard shortcuts window.
    Hotkeys,
    /// The project page on GitHub.
    ProjectPage,
}

/// A menu bar item was picked. Every window hears it (the UI listens globally); only `window` acts on it.
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[tauri_specta(event_name = "app-menu://action")]
pub struct AppMenuPicked {
    /// What to do.
    pub action: AppMenuAction,
    /// Label of the window in front (`main` when none is).
    pub window: String,
}

const ID_PREFIX: &str = "wave-menu:";
const MAX_TEXT: usize = 80;

fn clip(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(MAX_TEXT)
        .collect()
}

fn item_id(a: AppMenuAction) -> String {
    let name = serde_json::to_value(a)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    format!("{ID_PREFIX}{name}")
}

fn parse_id(id: &str) -> Option<AppMenuAction> {
    let name = id.strip_prefix(ID_PREFIX)?;
    serde_json::from_value(serde_json::Value::String(name.to_owned())).ok()
}

/// Builds and sets the menu bar (macOS); elsewhere does nothing.
pub fn apply(app: &AppHandle, labels: &AppMenuLabels) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let menu = build(app, labels)?;
        app.set_menu(menu)?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, labels);
    Ok(())
}

#[cfg(target_os = "macos")]
fn build(app: &AppHandle, l: &AppMenuLabels) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
    use AppMenuAction as A;

    let item = |a: A, text: &str, accel: Option<&str>| {
        MenuItem::with_id(app, item_id(a), clip(text), true, accel)
    };
    let sep = || PredefinedMenuItem::separator(app);
    let text = |s: &str| Some(clip(s));

    let about = PredefinedMenuItem::about(
        app,
        text(&l.about).as_deref(),
        Some(AboutMetadata {
            name: Some("Wave Shell".into()),
            version: Some(app.package_info().version.to_string()),
            ..Default::default()
        }),
    )?;
    let wave = Submenu::with_items(
        app,
        "Wave Shell",
        true,
        &[
            &about,
            &item(A::CheckUpdate, &l.check_update, None)?,
            &sep()?,
            &item(A::Settings, &l.settings, Some("CmdOrCtrl+,"))?,
            &item(A::Keys, &l.keys, None)?,
            &sep()?,
            &PredefinedMenuItem::services(app, text(&l.services).as_deref())?,
            &sep()?,
            &PredefinedMenuItem::hide(app, text(&l.hide).as_deref())?,
            &PredefinedMenuItem::hide_others(app, text(&l.hide_others).as_deref())?,
            &PredefinedMenuItem::show_all(app, text(&l.show_all).as_deref())?,
            &sep()?,
            &PredefinedMenuItem::quit(app, text(&l.quit).as_deref())?,
        ],
    )?;
    let file = Submenu::with_items(
        app,
        clip(&l.file),
        true,
        &[
            &item(A::NewTab, &l.new_tab, Some("CmdOrCtrl+T"))?,
            &item(A::LocalTerminal, &l.local_terminal, None)?,
            &item(A::NewHost, &l.new_host, None)?,
            &sep()?,
            &item(A::CloseTab, &l.close_tab, Some("CmdOrCtrl+W"))?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        clip(&l.edit),
        true,
        &[
            &PredefinedMenuItem::undo(app, text(&l.undo).as_deref())?,
            &PredefinedMenuItem::redo(app, text(&l.redo).as_deref())?,
            &sep()?,
            &PredefinedMenuItem::cut(app, text(&l.cut).as_deref())?,
            &PredefinedMenuItem::copy(app, text(&l.copy).as_deref())?,
            &PredefinedMenuItem::paste(app, text(&l.paste).as_deref())?,
            &PredefinedMenuItem::select_all(app, text(&l.select_all).as_deref())?,
        ],
    )?;
    let view = Submenu::with_items(
        app,
        clip(&l.view),
        true,
        &[
            &item(A::ToggleSidebar, &l.toggle_sidebar, Some("CmdOrCtrl+B"))?,
            &sep()?,
            &item(A::ScaleIn, &l.scale_in, Some("CmdOrCtrl+Shift+="))?,
            &item(A::ScaleOut, &l.scale_out, Some("CmdOrCtrl+Shift+-"))?,
            &item(A::ScaleReset, &l.scale_reset, Some("CmdOrCtrl+Shift+0"))?,
            &sep()?,
            &PredefinedMenuItem::fullscreen(app, text(&l.fullscreen).as_deref())?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        clip(&l.window),
        true,
        &[
            &PredefinedMenuItem::minimize(app, text(&l.minimize).as_deref())?,
            &PredefinedMenuItem::maximize(app, text(&l.zoom).as_deref())?,
        ],
    )?;
    let help = Submenu::with_items(
        app,
        clip(&l.help),
        true,
        &[
            &item(A::Hotkeys, &l.hotkeys, None)?,
            &item(A::ProjectPage, &l.project_page, None)?,
        ],
    )?;
    let menu = Menu::with_items(app, &[&wave, &file, &edit, &view, &window, &help])?;
    // The system adds its own entries: the open windows to «Окно», the search field to «Справка».
    window.set_as_windows_menu_for_nsapp()?;
    help.set_as_help_menu_for_nsapp()?;
    Ok(menu)
}

/// Sets the menu bar with the given texts (macOS; elsewhere nothing). Sync on purpose: runs on the main
/// thread, where native menus live. Passive: the UI calls it at start, not the user.
#[tauri::command]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)] // Tauri injects the handle by value.
pub fn app_menu_set(app: AppHandle, labels: AppMenuLabels) -> crate::api::ApiResult<()> {
    apply(&app, &labels).map_err(|_| crate::api::ApiError::Internal("menu unavailable".into()))
}

/// A menu item was picked: an app item is meant for the window in front (or the main one).
pub fn on_event(app: &AppHandle, id: &str) {
    use tauri::Manager;
    use tauri_specta::Event;

    let Some(action) = parse_id(id) else { return };
    let window = app
        .webview_windows()
        .into_iter()
        .find(|(_, w)| w.is_focused().unwrap_or(false))
        .map_or_else(|| "main".to_owned(), |(label, _)| label);
    let _ = AppMenuPicked { action, window }.emit(app);
}

#[cfg(test)]
mod tests {
    use super::{item_id, parse_id, AppMenuAction};

    #[test]
    fn ids_round_trip_and_foreign_ids_are_ignored() {
        for a in [
            AppMenuAction::CheckUpdate,
            AppMenuAction::CloseTab,
            AppMenuAction::ScaleReset,
            AppMenuAction::ProjectPage,
        ] {
            assert_eq!(parse_id(&item_id(a)), Some(a));
        }
        assert_eq!(item_id(AppMenuAction::NewTab), "wave-menu:new_tab");
        // The tray menu shares the global menu events: its ids are not ours.
        for foreign in ["quit", "open", "host:1", "wave-menu:", "wave-menu:nope"] {
            assert_eq!(parse_id(foreign), None);
        }
    }
}
