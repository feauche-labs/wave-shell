/**
 * Wave: the macOS menu bar in the app's language (see `src-tauri/src/wave/app_menu.rs`). The main window sends
 * the texts once at start — a language change reloads the page, so it sends them again. The app items come
 * back as `appMenuAction`; every window hears it, only the one named in it acts.
 */
import { api, apiEvents, type AppMenuAction, type AppMenuLabels } from '$lib/api';
import * as m from '$lib/paraglide/messages.js';

/** «Справка → Страница проекта на GitHub». */
export const PROJECT_PAGE = 'https://github.com/feauche-labs/wave-shell';

/** Every text of the menu bar in the current language. */
export function menuLabels(): AppMenuLabels {
  return {
    about: m.wave_menu_about(),
    check_update: m.wave_menu_check_update(),
    settings: m.wave_menu_settings(),
    keys: m.wave_menu_keys(),
    services: m.wave_menu_services(),
    hide: m.wave_menu_hide(),
    hide_others: m.wave_menu_hide_others(),
    show_all: m.wave_menu_show_all(),
    quit: m.wave_menu_quit(),
    file: m.wave_menu_file(),
    new_tab: m.wave_menu_new_tab(),
    local_terminal: m.wave_menu_local_terminal(),
    new_host: m.wave_menu_new_host(),
    close_tab: m.wave_menu_close_tab(),
    edit: m.wave_menu_edit(),
    undo: m.wave_menu_undo(),
    redo: m.wave_menu_redo(),
    cut: m.wave_menu_cut(),
    copy: m.wave_menu_copy(),
    paste: m.wave_menu_paste(),
    select_all: m.wave_menu_select_all(),
    view: m.wave_menu_view(),
    toggle_sidebar: m.wave_menu_toggle_sidebar(),
    scale_in: m.wave_keys_scale_in(),
    scale_out: m.wave_keys_scale_out(),
    scale_reset: m.wave_keys_scale_reset(),
    fullscreen: m.wave_menu_fullscreen(),
    window: m.wave_menu_window(),
    minimize: m.wave_menu_minimize(),
    zoom: m.wave_menu_zoom(),
    help: m.wave_menu_help(),
    hotkeys: m.keys_title(),
    project_page: m.wave_menu_project_page()
  };
}

/** Sends the menu texts (quietly: the menu bar is not worth an error toast). */
export function sendMenu(): void {
  void api.appMenuSet(menuLabels()).catch(() => undefined);
}

/** Runs `run` for the menu items picked while this window (`label`) is in front. */
export function onMenuAction(
  label: string,
  run: (action: AppMenuAction) => void
): Promise<() => void> {
  return apiEvents.listen('appMenuAction', (p) => {
    if (p.window === label) run(p.action);
  });
}
