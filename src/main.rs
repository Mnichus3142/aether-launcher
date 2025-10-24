use anyhow::Result;
use gio::prelude::*;
use glib::Propagation;
use gtk4::prelude::*;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::{Application, ApplicationWindow, EventControllerKey};
use gtk4::gdk;
use gtk4::CssProvider;
use gtk4_layer_shell as gls;
use gls::LayerShell;
use gls::KeyboardMode;
use std::path::PathBuf;
use gio::File;
use webkit6::prelude::*;
use std::fs;
use webkit6::WebView;

const APP_ID: &str = "dev.example.aether-launcher";
const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;

fn find_static_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
    }
    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("static")
    }
}


fn build_bar(app: &Application) -> ApplicationWindow {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("Aether-launcher")
        .resizable(false)
        .modal(false)
        .build();

    let provider = CssProvider::new();
    let _ = provider.load_from_data("window, .background { background-color: transparent; }");

    win.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    win.set_can_focus(true);
    
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let is_wayland_display = display.type_().name().to_ascii_lowercase().contains("wayland");
        
        if is_wayland_display && gls::is_supported() {
            win.init_layer_shell();
            win.set_layer(gls::Layer::Top);
            
            win.set_anchor(gls::Edge::Left, false);
            win.set_anchor(gls::Edge::Top, false); 
            win.set_anchor(gls::Edge::Right, false);
            win.set_anchor(gls::Edge::Bottom, false);
            
            win.set_size_request(WINDOW_WIDTH, WINDOW_HEIGHT);

            win.auto_exclusive_zone_enable();

            win.set_keyboard_mode(KeyboardMode::OnDemand);
            
            if let Some(monitor) = display.monitors().item(0).and_then(|obj| obj.downcast::<gdk::Monitor>().ok()) {
                let geometry = monitor.geometry();
                
                let center_x = geometry.width() / 2;
                let center_y = geometry.height() / 2;
                let left_margin = center_x - WINDOW_WIDTH / 2;
                let top_margin = center_y - WINDOW_HEIGHT / 2;
                
                win.set_margin(gls::Edge::Left, left_margin.max(0));
                win.set_margin(gls::Edge::Top, top_margin.max(0));
            }
        }
    }

    let webview = WebView::new();
    if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
        settings.set_enable_javascript(true);
        settings.set_enable_developer_extras(true);
        settings.set_allow_universal_access_from_file_urls(true);
        settings.set_allow_file_access_from_file_urls(true);
    }

    let rgba = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
    webview.set_background_color(&rgba);
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    
    webview.set_can_focus(true);
    webview.set_focusable(true);
    
    let static_dir = find_static_dir();

    let rgba = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
    webview.set_background_color(&rgba);

    let user_config_dir = glib::user_config_dir().join("aether-launcher");
    let user_css_path = user_config_dir.join("style.css");
    let mut user_css_exists = user_css_path.exists();

    if !user_css_exists {
        let default_css_path = static_dir.join("styles.css");
        match fs::read_to_string(&default_css_path) {
            Ok(default_css) => {
                if let Err(e) = fs::create_dir_all(&user_config_dir) {
                    eprintln!("[Aether-config] Could not create config directory: {}", e);
                }
                match fs::write(&user_css_path, default_css) {
                    Ok(_) => {
                        eprintln!("[Aether-config] Created default ~/.config/aether-launcher/style.css");
                        user_css_exists = true;
                    }
                    Err(e) => eprintln!("[Aether-config] Could not save user style: {}", e),
                }
            }
            Err(e) => eprintln!("[Aether-config] Could not read default CSS: {}", e),
        }
    }

    let user_css_uri: String = File::for_path(&user_css_path).uri().to_string();

    let user_config_json_path = user_config_dir.join("config.json");
    if !user_config_json_path.exists() {
        let default_cfg_path = static_dir.join("config.json");
        match fs::read_to_string(&default_cfg_path) {
            Ok(default_cfg) => {
                if let Err(e) = fs::create_dir_all(&user_config_dir) {
                    eprintln!("[Aether-config] Could not create config directory: {}", e);
                }
                if let Err(e) = fs::write(&user_config_json_path, default_cfg) {
                    eprintln!("[Aether-config] Could not save user config: {}", e);
                } else {
                    eprintln!("[Aether-config] Created default ~/.config/aether-launcher/config.json");
                }
            }
            Err(e) => eprintln!("[Aether-config] Could not read default config.json: {}", e),
        }
    }

    let mut injected_config_json = String::from("{}");
    if let Ok(cfg) = fs::read_to_string(&user_config_json_path) {
        injected_config_json = cfg;
    }

    let index_path = static_dir.join("index.html");
    let index_uri = File::for_path(&index_path).uri();
    eprintln!("[Aether-config] Loading URI: {}", index_uri);

    {
        let user_css_uri = user_css_uri.clone();
        let user_css_exists_captured = user_css_exists;
        let injected_config_json = injected_config_json.clone();
        webview.connect_load_changed(move |wv, ev| {
            eprintln!("[Aether-config] WebView load_changed: {:?}", ev);
            if matches!(ev, webkit6::LoadEvent::Committed | webkit6::LoadEvent::Finished) {
                if user_css_exists_captured {
                    let js = format!(
                        r#"(function(){{
  try {{
    const head = document.head || document.getElementsByTagName('head')[0];
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.type = 'text/css';
    link.href = '{}';
    head.appendChild(link);
  }} catch (e) {{
    console.error('Inject CSS failed', e);
  }}
}})();"#,
                        user_css_uri.replace('\'', "\\'")
                    );
                    wv.evaluate_javascript(
                        &js,
                        None::<&str>,
                        None::<&str>,
                        None::<&gtk4::gio::Cancellable>,
                        |_| {},
                    );
                }

                let js_cfg = format!(
                    r#"(function(){{
  try {{
    window.AetherConfig = Object.freeze({});
    window.dispatchEvent(new CustomEvent('config', {{ detail: window.Aether-LauncherConfig }}));
  }} catch (e) {{ console.error('Inject config failed', e); }}
}})();"#,
                    injected_config_json
                );

                wv.evaluate_javascript(
                    &js_cfg,
                    None::<&str>,
                    None::<&str>,
                    None::<&gtk4::gio::Cancellable>,
                    |_| {},
                );
            }
        });
    }

    webview.load_uri(&index_uri);
    
    webview.connect_load_changed(move |webview, _| {
        webview.grab_focus();
    });
    
    win.set_child(Some(&webview));

    let key_controller = EventControllerKey::new();
    let win_clone = win.clone();
    
    key_controller.connect_key_pressed(move |_, key, _code, _modifier| {
        if key == gdk::Key::Escape {
            win_clone.close();
            return Propagation::Stop;
        }
        Propagation::Proceed
    });
    
    win.add_controller(key_controller);

    win
}

fn main() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let win = build_bar(app);
        win.present();
    });
    app.run();
    Ok(())
}