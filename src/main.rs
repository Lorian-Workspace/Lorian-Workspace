// Lorian Workspace - Linux Adaptation
// Discord Rich Presence con rotación automática de actividades
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use crossbeam_channel::{unbounded, Receiver, Sender};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[cfg(windows)]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Console::*, LibraryLoader::*},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

// Alias para evitar conflictos con windows::core::Result
type StdResult<T, E> = std::result::Result<T, E>;

// ============================================================
// Data structures
// ============================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct ButtonConfig {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivityConfig {
    pub name: String,
    pub details: String,
    pub state: String,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub duration_seconds: u64,
    pub buttons: Option<Vec<ButtonConfig>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordConfig {
    pub app_id: String,
    pub activities: Vec<ActivityConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    Pause,
    Resume,
    NextActivity,
    ReloadConfig,
    ShowStatus,
    ToggleConsole,
    OpenConfig,
    Exit,
}

#[derive(Debug)]
pub struct AppState {
    pub is_paused: bool,
    pub is_running: bool,
}

// ============================================================
// Discord Presence Manager
// ============================================================

#[derive(Debug)]
pub struct DiscordPresenceManager {
    client: Option<DiscordIpcClient>,
    app_id: String,
    pub activities: Vec<ActivityConfig>,
    current_activity_index: usize,
    is_connected: bool,
    last_connection_attempt: SystemTime,
}

impl DiscordPresenceManager {
    pub fn new(app_id: &str, activities: Vec<ActivityConfig>) -> Self {
        Self {
            client: None,
            app_id: app_id.to_string(),
            activities,
            current_activity_index: 0,
            is_connected: false,
            last_connection_attempt: SystemTime::UNIX_EPOCH,
        }
    }

    pub async fn connect(&mut self) -> StdResult<(), String> {
        self.last_connection_attempt = SystemTime::now();

        let mut client = DiscordIpcClient::new(&self.app_id)
            .map_err(|e| format!("Error creando cliente Discord: {}", e))?;
        client
            .connect()
            .map_err(|e| format!("Error conectando a Discord: {}", e))?;

        log_info("Discord RPC conectado exitosamente!");
        self.client = Some(client);
        self.is_connected = true;
        Ok(())
    }

    pub async fn try_reconnect(&mut self) -> bool {
        if let Ok(elapsed) = self.last_connection_attempt.elapsed() {
            if elapsed.as_secs() < 5 {
                return false;
            }
        }

        log_info("🔄 Intentando reconectar a Discord...");
        match self.connect().await {
            Ok(_) => {
                log_info("✅ Reconexión exitosa!");
                if let Err(e) = self.set_current_activity().await {
                    log_error(&format!("⚠️  Error restaurando actividad: {}", e));
                }
                true
            }
            Err(e) => {
                log_error(&format!("❌ Fallo la reconexión: {}", e));
                false
            }
        }
    }

    pub fn is_connection_alive(&self) -> bool {
        self.is_connected && self.client.is_some()
    }

    pub async fn set_current_activity(&mut self) -> StdResult<(), String> {
        if let Some(client) = &mut self.client {
            if self.activities.is_empty() {
                return Ok(());
            }

            let activity_config = &self.activities[self.current_activity_index];
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut activity_builder = activity::Activity::new()
                .details(&activity_config.details)
                .state(&activity_config.state)
                .timestamps(activity::Timestamps::new().start(timestamp as i64));

            if let Some(large_image) = &activity_config.large_image {
                let mut assets = activity::Assets::new().large_image(large_image);

                if let Some(large_text) = &activity_config.large_text {
                    assets = assets.large_text(large_text);
                }

                if let Some(small_image) = &activity_config.small_image {
                    assets = assets.small_image(small_image);

                    if let Some(small_text) = &activity_config.small_text {
                        assets = assets.small_text(small_text);
                    }
                }

                activity_builder = activity_builder.assets(assets);
            }

            let activity = if let Some(buttons) = &activity_config.buttons {
                let discord_buttons = buttons
                    .iter()
                    .map(|btn| activity::Button::new(&btn.label, &btn.url))
                    .collect::<Vec<_>>();

                activity_builder.buttons(discord_buttons)
            } else {
                activity_builder
            };

            match client.set_activity(activity) {
                Ok(_) => {
                    self.is_connected = true;
                    log_info(&format!(
                        "🎯 Actividad '{}' activada: {} - {} (por {} segundos)",
                        activity_config.name,
                        activity_config.details,
                        activity_config.state,
                        activity_config.duration_seconds
                    ));
                }
                Err(e) => {
                    let error_msg = format!("Error enviando actividad a Discord: {}", e);
                    log_error(&error_msg);
                    self.is_connected = false;
                    self.client = None;
                    return Err(error_msg);
                }
            }
        }
        Ok(())
    }

    pub fn get_current_activity_duration(&self) -> u64 {
        if self.activities.is_empty() {
            return 30;
        }
        self.activities[self.current_activity_index].duration_seconds
    }

    pub fn next_activity(&mut self) {
        if !self.activities.is_empty() {
            self.current_activity_index = (self.current_activity_index + 1) % self.activities.len();
        }
    }

    pub fn reload_activities(&mut self, new_activities: Vec<ActivityConfig>) {
        self.activities = new_activities;
        if self.current_activity_index >= self.activities.len() {
            self.current_activity_index = 0;
        }
        log_info(&format!(
            "🔄 Actividades recargadas: {} disponibles",
            self.activities.len()
        ));
    }

    pub fn get_status(&self) -> String {
        if self.activities.is_empty() {
            return "❌ Sin actividades configuradas".to_string();
        }

        let current = &self.activities[self.current_activity_index];
        format!(
            "📊 Actividad actual: {} ({}/{}) - {}",
            current.name,
            self.current_activity_index + 1,
            self.activities.len(),
            current.details
        )
    }

    pub async fn clear_activity(&mut self) -> StdResult<(), String> {
        if let Some(client) = &mut self.client {
            client
                .clear_activity()
                .map_err(|e| format!("Error limpiando actividad: {}", e))?;
            log_info("Actividad de Discord limpiada");
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
            println!("Discord RPC desconectado");
        }
        self.is_connected = false;
        self.client = None;
    }
}

// ============================================================
// File system helpers
// ============================================================

pub fn get_app_data_dir() -> StdResult<PathBuf, String> {
    #[cfg(windows)]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            let app_dir = PathBuf::from(appdata).join("lorianworkspace");
            return Ok(app_dir);
        }
    }

    if let Ok(home) = env::var("HOME") {
        let app_dir = PathBuf::from(home).join(".lorianworkspace");
        return Ok(app_dir);
    }

    Ok(PathBuf::from("."))
}

pub fn ensure_app_data_dir_exists() -> StdResult<PathBuf, String> {
    let app_dir = get_app_data_dir()?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Error creando directorio {}: {}", app_dir.display(), e))?;
        println!("📁 Creado directorio: {}", app_dir.display());
    }

    Ok(app_dir)
}

pub fn get_default_config() -> Config {
    Config {
        discord: DiscordConfig {
            app_id: "1234567890123456789".to_string(),
            activities: vec![
                ActivityConfig {
                    name: "working".to_string(),
                    details: "🎨 Working on a project".to_string(),
                    state: "In the zone".to_string(),
                    large_image: Some("main_logo".to_string()),
                    large_text: Some("Lorian Workspace".to_string()),
                    small_image: Some("status_online".to_string()),
                    small_text: Some("Online".to_string()),
                    duration_seconds: 30,
                    buttons: Some(vec![ButtonConfig {
                        label: "🌐 Visit Website".to_string(),
                        url: "https://example.com".to_string(),
                    }]),
                },
            ],
        },
    }
}

pub fn create_default_config_file(config_path: &Path) -> StdResult<(), String> {
    let default_config = get_default_config();
    let config_json = serde_json::to_string_pretty(&default_config)
        .map_err(|e| format!("Error serializando configuración default: {}", e))?;

    fs::write(config_path, config_json)
        .map_err(|e| format!("Error escribiendo archivo de configuración: {}", e))?;

    log_info(&format!(
        "📝 Creado archivo de configuración default: {}",
        config_path.display()
    ));
    Ok(())
}

pub fn load_config() -> StdResult<Config, String> {
    let app_dir = ensure_app_data_dir_exists()?;
    let config_path = app_dir.join("config.json");

    if !config_path.exists() {
        log_info("🆕 Primera ejecución - creando configuración default...");
        create_default_config_file(&config_path)?;
        log_info(&format!(
            "📍 Configuración guardada en: {}",
            config_path.display()
        ));
        log_info("💡 Puedes editar este archivo para personalizar tu app");
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Error leyendo {}: {}", config_path.display(), e))?;
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Error parseando JSON en {}: {}", config_path.display(), e))?;

    Ok(config)
}

pub fn setup_file_watcher(
    command_sender: Sender<AppCommand>,
) -> StdResult<RecommendedWatcher, notify::Error> {
    let mut watcher = RecommendedWatcher::new(
        move |res: StdResult<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in event.paths {
                            if path.file_name().and_then(|n| n.to_str()) == Some("config.json") {
                                log_info("📁 Detectado cambio en config.json - recargando...");
                                let _ = command_sender.send(AppCommand::ReloadConfig);
                                break;
                            }
                        }
                    }
                }
                Err(e) => log_error(&format!("⚠️  Error del file watcher: {:?}", e)),
            }
        },
        NotifyConfig::default(),
    )?;

    match get_app_data_dir() {
        Ok(app_dir) => {
            watcher.watch(&app_dir, RecursiveMode::NonRecursive)?;
            log_info(&format!(
                "👁️  File watcher iniciado para: {}",
                app_dir.join("config.json").display()
            ));
        }
        Err(e) => {
            log_error(&format!(
                "⚠️  Error obteniendo directorio de AppData: {}",
                e
            ));
            watcher.watch(Path::new("."), RecursiveMode::NonRecursive)?;
            log_info("👁️  File watcher iniciado en directorio actual");
        }
    }

    Ok(watcher)
}

// ============================================================
// System Tray (Windows only) / Placeholder (Linux)
// ============================================================

#[cfg(windows)]
mod tray {
    use super::*;
    use std::mem::size_of;

    const WM_TRAYICON: u32 = WM_USER + 1;
    const ID_TRAY_ICON: u32 = 1;
    const ID_MENU_PAUSE: u32 = 1001;
    const ID_MENU_RESUME: u32 = 1002;
    const ID_MENU_NEXT: u32 = 1003;
    const ID_MENU_RELOAD: u32 = 1004;
    const ID_MENU_STATUS: u32 = 1005;
    const ID_MENU_TOGGLE_CONSOLE: u32 = 1006;
    const ID_MENU_OPEN_CONFIG: u32 = 1007;
    const ID_MENU_EXIT: u32 = 1008;

    fn loword(l: u32) -> u16 {
        (l & 0xFFFF) as u16
    }

    static mut GLOBAL_COMMAND_SENDER: Option<Sender<AppCommand>> = None;

    pub struct SystemTray {
        hwnd: HWND,
        command_sender: Sender<AppCommand>,
    }

    impl SystemTray {
        pub fn new(command_sender: Sender<AppCommand>) -> windows::core::Result<Self> {
            unsafe {
                GLOBAL_COMMAND_SENDER = Some(command_sender.clone());
                let instance = GetModuleHandleW(None)?;
                let window_class = WNDCLASSEXW {
                    cbSize: size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(Self::window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: instance.into(),
                    hIcon: LoadIconW(None, IDI_APPLICATION)?,
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: HBRUSH(std::ptr::null_mut()),
                    lpszMenuName: PCWSTR::null(),
                    lpszClassName: w!("LorianWorkspace"),
                    hIconSm: HICON(std::ptr::null_mut()),
                };
                RegisterClassExW(&window_class);
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("LorianWorkspace"),
                    w!("Lorian Workspace"),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT, CW_USEDEFAULT,
                    CW_USEDEFAULT, CW_USEDEFAULT,
                    None, None, instance, None,
                )?;
                let tray = Self { hwnd, command_sender };
                tray.add_tray_icon()?;
                Ok(tray)
            }
        }

        unsafe fn add_tray_icon(&self) -> windows::core::Result<()> {
            let icon = LoadIconW(None, IDI_INFORMATION).unwrap_or_default();
            let mut nid = NOTIFYICONDATAW {
                cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: ID_TRAY_ICON,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: icon,
                szTip: [0; 128],
                ..Default::default()
            };
            let tip = "Lorian Workspace";
            let tip_wide: Vec<u16> = tip.encode_utf16().collect();
            nid.szTip[..tip_wide.len().min(127)].copy_from_slice(&tip_wide[..tip_wide.len().min(127)]);
            Shell_NotifyIconW(NIM_ADD, &nid);
            Ok(())
        }

        unsafe extern "system" fn window_proc(
            hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
        ) -> LRESULT {
            match msg {
                WM_TRAYICON => {
                    let event = loword(lparam.0 as u32) as u32;
                    match event {
                        WM_RBUTTONUP => {
                            let hmenu = CreatePopupMenu().unwrap();
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_PAUSE as usize, w!("⏸️ Pausar"));
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_NEXT as usize, w!("⏭️ Siguiente"));
                            AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_OPEN_CONFIG as usize, w!("📝 Config"));
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_RELOAD as usize, w!("🔄 Recargar"));
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_STATUS as usize, w!("📊 Estado"));
                            AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                            AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT as usize, w!("❌ Salir"));
                            let mut pt = POINT { x: 0, y: 0 };
                            GetCursorPos(&mut pt);
                            SetForegroundWindow(hwnd);
                            let cmd = TrackPopupMenu(hmenu, TPM_RIGHTBUTTON | TPM_RETURNCMD, pt.x, pt.y, 0, hwnd, None);
                            if cmd.as_bool() {
                                let command = match cmd.0 as u32 {
                                    ID_MENU_PAUSE => AppCommand::Pause,
                                    ID_MENU_NEXT => AppCommand::NextActivity,
                                    ID_MENU_RELOAD => AppCommand::ReloadConfig,
                                    ID_MENU_STATUS => AppCommand::ShowStatus,
                                    ID_MENU_OPEN_CONFIG => AppCommand::OpenConfig,
                                    ID_MENU_EXIT => AppCommand::Exit,
                                    _ => return LRESULT(0),
                                };
                                if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                                    let _ = sender.send(command);
                                }
                            }
                            DestroyMenu(hmenu);
                        }
                        WM_LBUTTONDBLCLK => {
                            if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                                let _ = sender.send(AppCommand::ShowStatus);
                            }
                        }
                        _ => {}
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }

        pub fn process_messages(&self, _app_state: Arc<Mutex<AppState>>) -> bool {
            unsafe {
                let mut msg = MSG::default();
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        return false;
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                true
            }
        }
    }

    impl Drop for SystemTray {
        fn drop(&mut self) {
            unsafe {
                let nid = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: ID_TRAY_ICON,
                    ..Default::default()
                };
                Shell_NotifyIconW(NIM_DELETE, &nid);
            }
        }
    }
}

#[cfg(not(windows))]
mod tray {
    use super::*;

    pub struct SystemTray;

    impl SystemTray {
        pub fn new(_: Sender<AppCommand>) -> StdResult<Self, Box<dyn std::error::Error>> {
            Ok(SystemTray)
        }
    }
}

// ============================================================
// Console helpers (stubs on Linux)
// ============================================================

#[cfg(windows)]
fn hide_console() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            let _ = ShowWindow(console_window, SW_HIDE);
        }
    }
}

#[cfg(windows)]
fn show_console() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            let _ = ShowWindow(console_window, SW_SHOW);
            let _ = SetForegroundWindow(console_window);
        } else if AllocConsole().is_ok() {
            let new_console = GetConsoleWindow();
            if !new_console.is_invalid() {
                let _ = ShowWindow(new_console, SW_SHOW);
                let _ = SetForegroundWindow(new_console);
            }
        }
    }
}

#[cfg(windows)]
fn toggle_console() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            if IsWindowVisible(console_window).as_bool() {
                let _ = ShowWindow(console_window, SW_HIDE);
            } else {
                let _ = ShowWindow(console_window, SW_SHOW);
                let _ = SetForegroundWindow(console_window);
            }
        } else {
            show_console();
        }
    }
}

#[cfg(windows)]
fn open_config_file() {
    use windows::Win32::UI::Shell::*;
    if let Ok(app_dir) = get_app_data_dir() {
        let config_path = app_dir.join("config.json");
        if config_path.exists() {
            let path_str = config_path.to_string_lossy();
            let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                let _ = ShellExecuteW(None, None, PCWSTR(path_wide.as_ptr()), None, None, SW_SHOW);
            }
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn hide_console() {}

#[cfg(not(windows))]
#[allow(dead_code)]
fn show_console() {}

#[cfg(not(windows))]
#[allow(dead_code)]
fn toggle_console() {}

#[cfg(not(windows))]
#[allow(dead_code)]
fn open_config_file() {}

// ============================================================
// Logging
// ============================================================

fn get_log_file_path() -> StdResult<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("app.log"))
}

fn log_message(message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    let timestamp_str = format!("{:?}", datetime);

    let log_entry = format!("[{}] {}\n", timestamp_str, message);

    if let Ok(log_path) = get_log_file_path() {
        if let Ok(metadata) = fs::metadata(&log_path) {
            if metadata.len() > 5_242_880 {
                let _ = fs::remove_file(&log_path);
                let rotation_msg = format!("[{}] 🔄 Log rotado - archivo anterior borrado (>5MB)\n", timestamp_str);
                let _ = fs::write(&log_path, rotation_msg);
            }
        }

        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }

    // Siempre mostrar en stdout en Linux
    print!("{}", log_entry);
    let _ = io::stdout().flush();
}

fn log_error(message: &str) {
    log_message(&format!("ERROR: {}", message));
}

fn log_info(message: &str) {
    log_message(&format!("INFO: {}", message));
}

// ============================================================
// CLI command handler for Linux (stdin)
// ============================================================

#[cfg(not(windows))]
async fn run_stdin_reader(command_sender: Sender<AppCommand>) {
    let stdin = io::stdin();
    let reader = stdin.lock();
    log_info("⌨️  Comandos disponibles: pause, resume, next, status, reload, exit");
    log_info("💡 Escribe un comando y presiona Enter");

    for line in reader.lines() {
        match line {
            Ok(input) => {
                let cmd = input.trim().to_lowercase();
                match cmd.as_str() {
                    "pause" | "p" => {
                        let _ = command_sender.send(AppCommand::Pause);
                    }
                    "resume" | "r" => {
                        let _ = command_sender.send(AppCommand::Resume);
                    }
                    "next" | "n" => {
                        let _ = command_sender.send(AppCommand::NextActivity);
                    }
                    "status" | "s" => {
                        let _ = command_sender.send(AppCommand::ShowStatus);
                    }
                    "reload" => {
                        let _ = command_sender.send(AppCommand::ReloadConfig);
                    }
                    "exit" | "quit" | "q" => {
                        let _ = command_sender.send(AppCommand::Exit);
                        break;
                    }
                    "" => {}
                    _ => {
                        println!("Comando desconocido: '{}'", cmd);
                        println!("Comandos: pause, resume, next, status, reload, exit");
                    }
                }
            }
            Err(e) => {
                log_error(&format!("Error leyendo stdin: {}", e));
                break;
            }
        }
    }
}

#[cfg(not(windows))]
fn print_help() {
    println!("Lorian Workspace - Discord Rich Presence Rotator");
    println!("");
    println!("Uso: lorianworkspace [COMANDO]");
    println!("");
    println!("Comandos (opcional, para control único):");
    println!("  --pause, -p       Pausar rotación de actividades");
    println!("  --resume, -r      Reanudar rotación");
    println!("  --next, -n        Siguiente actividad");
    println!("  --status, -s      Mostrar estado actual");
    println!("  --help, -h        Mostrar esta ayuda");
    println!("");
    println!("Sin argumentos, corre en modo interactivo.");
    println!("Comandos disponibles en stdin: pause, resume, next, status, reload, exit");
}

#[cfg(windows)]
fn print_help() {}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() -> StdResult<(), String> {
    let args: Vec<String> = env::args().collect();

    // Parse arg for one-shot commands (Linux)
    #[cfg(not(windows))]
    {
        if args.len() > 1 {
            let oneshot = args[1].as_str();
            match oneshot {
                "--help" | "-h" => {
                    print_help();
                    return Ok(());
                }
                _ => {
                    // One-shot commands handled via pipe/socket in future
                    print_help();
                    return Ok(());
                }
            }
        }
    }

    log_info("🚀 Iniciando Lorian Workspace...");
    log_info("🐧 Modo Linux - Rich Presence de Discord");

    // Mostrar info del log
    if let Ok(log_path) = get_log_file_path() {
        log_info(&format!("📝 Logs guardados en: {}", log_path.display()));
        log_info("🔄 Rotación automática: logs se borran al superar 5MB");
        if let Ok(metadata) = fs::metadata(&log_path) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            log_info(&format!("📊 Tamaño actual del log: {:.2}MB", size_mb));
        }
    }

    // Cargar configuración
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log_error(&format!("❌ Error cargando config.json: {}", e));
            return Err(e);
        }
    };

    if config.discord.activities.is_empty() {
        log_error("⚠️  No hay actividades configuradas en config.json");
        return Ok(());
    }

    log_info(&format!(
        "📋 Cargadas {} actividades desde config.json",
        config.discord.activities.len()
    ));

    let discord_manager = Arc::new(Mutex::new(DiscordPresenceManager::new(
        &config.discord.app_id,
        config.discord.activities,
    )));

    let app_state = Arc::new(Mutex::new(AppState {
        is_paused: false,
        is_running: true,
    }));

    let (command_sender, command_receiver): (Sender<AppCommand>, Receiver<AppCommand>) =
        unbounded();

    // Conectar a Discord
    {
        let mut manager = discord_manager.lock().await;
        match manager.connect().await {
            Ok(_) => {
                if let Err(e) = manager.set_current_activity().await {
                    log_error(&format!("⚠️  Error estableciendo actividad inicial: {}", e));
                } else {
                    log_info("✅ Discord Rich Presence activado con rotación automática!");
                }
            }
            Err(e) => {
                log_error(&format!("❌ Error conectando a Discord: {}", e));
                log_error("💡 Asegúrate de que Discord esté abierto y tengas un Application ID válido");
            }
        }
    }

    // Iniciar File Watcher para hot reload
    let _file_watcher = setup_file_watcher(command_sender.clone())
        .map_err(|e| format!("Error iniciando file watcher: {}", e))?;
    log_info("🔥 Hot reload activado - edita config.json y se recargará automáticamente");

    // Sistema de rotación de actividades con auto-reconexión
    let discord_manager_clone = discord_manager.clone();
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        loop {
            let (duration, is_paused, is_connected) = {
                let manager = discord_manager_clone.lock().await;
                let state = app_state_clone.lock().await;
                (
                    manager.get_current_activity_duration(),
                    state.is_paused,
                    manager.is_connection_alive(),
                )
            };

            if !is_connected {
                let mut manager = discord_manager_clone.lock().await;
                if manager.try_reconnect().await {
                    log_info("🔗 Reconexión exitosa - continuando rotación");
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;

            if !is_paused {
                let mut manager = discord_manager_clone.lock().await;
                if manager.is_connection_alive() {
                    manager.next_activity();
                    let _ = manager.set_current_activity().await;
                }
            }
        }
    });

    // Manejo de comandos
    let discord_manager_clone2 = discord_manager.clone();
    let app_state_clone2 = app_state.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(command) = command_receiver.recv() {
                match command {
                    AppCommand::Pause => {
                        let mut state = app_state_clone2.lock().await;
                        state.is_paused = !state.is_paused;
                        log_info(if state.is_paused { "⏸️  Actividades pausadas" } else { "▶️ Actividades reanudadas" });
                    }
                    AppCommand::Resume => {
                        let mut state = app_state_clone2.lock().await;
                        state.is_paused = false;
                        log_info("▶️ Actividades reanudadas");
                    }
                    AppCommand::NextActivity => {
                        let mut manager = discord_manager_clone2.lock().await;
                        if !manager.is_connection_alive() {
                            if !manager.try_reconnect().await {
                                log_error("❌ No se puede cambiar actividad - Discord no está conectado");
                                continue;
                            }
                        }
                        manager.next_activity();
                        if let Err(e) = manager.set_current_activity().await {
                            log_error(&format!("⚠️  Error cambiando actividad: {}", e));
                        } else {
                            log_info("⏭️ Cambiado a siguiente actividad");
                        }
                    }
                    AppCommand::ReloadConfig => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        match load_config() {
                            Ok(new_config) => {
                                let mut manager = discord_manager_clone2.lock().await;
                                manager.reload_activities(new_config.discord.activities);
                                if let Err(e) = manager.set_current_activity().await {
                                    log_error(&format!("⚠️  Error aplicando nueva configuración: {}", e));
                                } else {
                                    log_info("🔄 Configuración recargada exitosamente");
                                }
                            }
                            Err(e) => log_error(&format!("❌ Error recargando configuración: {}", e)),
                        }
                    }
                    AppCommand::ShowStatus => {
                        let manager = discord_manager_clone2.lock().await;
                        let state = app_state_clone2.lock().await;
                        log_info("📊 === Estado de la Aplicación ===");
                        log_info(&manager.get_status());
                        log_info(&format!("⏸️  Rotación: {}", if state.is_paused { "Pausada" } else { "Activa" }));
                        log_info(&format!("🔗 Conexión: {}", if manager.is_connection_alive() { "✅ Conectado" } else { "❌ Desconectado" }));
                    }
                    AppCommand::ToggleConsole => toggle_console(),
                    AppCommand::OpenConfig => open_config_file(),
                    AppCommand::Exit => {
                        let mut state = app_state_clone2.lock().await;
                        state.is_running = false;
                        log_info("🛑 Cerrando aplicación...");
                        break;
                    }
                }
            }
        }
    });

    // Iniciar stdin reader en Linux
    #[cfg(not(windows))]
    {
        let stdin_sender = command_sender.clone();
        tokio::spawn(async move {
            run_stdin_reader(stdin_sender).await;
        });
    }

    // Inicializar System Tray (no-op en Linux)
    let _tray = tray::SystemTray::new(command_sender.clone()).map_err(|e| {
        log_error(&format!("{}", e));
        String::new()
    })?;

    // Loop principal
    loop {
        #[cfg(windows)]
        {
            if !_tray.process_messages(app_state.clone()) {
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let state = app_state.lock().await;
        if !state.is_running {
            break;
        }
    }

    log_info("🛑 Limpiando recursos...");
    let mut manager = discord_manager.lock().await;
    let _ = manager.clear_activity().await;
    manager.disconnect().await;
    log_info("👋 App cerrada correctamente!");

    Ok(())
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- DiscordPresenceManager tests ---

    #[test]
    fn test_new_manager_has_no_client() {
        let manager = DiscordPresenceManager::new("test_id", vec![]);
        assert!(manager.client.is_none());
        assert!(!manager.is_connection_alive());
        assert_eq!(manager.current_activity_index, 0);
    }

    #[test]
    fn test_next_activity_cycles_through_activities() {
        let activities = vec![
            ActivityConfig {
                name: "a".into(), details: "d1".into(), state: "s1".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 10, buttons: None,
            },
            ActivityConfig {
                name: "b".into(), details: "d2".into(), state: "s2".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 20, buttons: None,
            },
            ActivityConfig {
                name: "c".into(), details: "d3".into(), state: "s3".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 30, buttons: None,
            },
        ];

        let mut manager = DiscordPresenceManager::new("test_id", activities);

        assert_eq!(manager.current_activity_index, 0);
        assert_eq!(manager.get_current_activity_duration(), 10);

        manager.next_activity();
        assert_eq!(manager.current_activity_index, 1);
        assert_eq!(manager.get_current_activity_duration(), 20);

        manager.next_activity();
        assert_eq!(manager.current_activity_index, 2);
        assert_eq!(manager.get_current_activity_duration(), 30);

        // Wrap around
        manager.next_activity();
        assert_eq!(manager.current_activity_index, 0);
        assert_eq!(manager.get_current_activity_duration(), 10);
    }

    #[test]
    fn test_next_activity_empty_list() {
        let mut manager = DiscordPresenceManager::new("test_id", vec![]);
        manager.next_activity(); // should not panic
        assert_eq!(manager.current_activity_index, 0);
    }

    #[test]
    fn test_next_activity_single_item() {
        let activities = vec![ActivityConfig {
            name: "only".into(), details: "d".into(), state: "s".into(),
            large_image: None, large_text: None, small_image: None, small_text: None,
            duration_seconds: 5, buttons: None,
        }];
        let mut manager = DiscordPresenceManager::new("test_id", activities);
        assert_eq!(manager.current_activity_index, 0);
        manager.next_activity();
        assert_eq!(manager.current_activity_index, 0); // stays at 0
    }

    #[test]
    fn test_reload_activities_replaces_and_resets_index() {
        let activities = vec![
            ActivityConfig {
                name: "old".into(), details: "d".into(), state: "s".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 5, buttons: None,
            },
        ];
        let mut manager = DiscordPresenceManager::new("test_id", activities);

        // Advance to index 1 (wraps to 0 since only 1 item)
        manager.next_activity();
        assert_eq!(manager.current_activity_index, 0);

        // Reload with new activities
        let new_activities = vec![
            ActivityConfig {
                name: "new1".into(), details: "d".into(), state: "s".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 10, buttons: None,
            },
            ActivityConfig {
                name: "new2".into(), details: "d".into(), state: "s".into(),
                large_image: None, large_text: None, small_image: None, small_text: None,
                duration_seconds: 20, buttons: None,
            },
        ];
        manager.reload_activities(new_activities);
        assert_eq!(manager.current_activity_index, 0);
        assert_eq!(manager.activities.len(), 2);
    }

    #[test]
    fn test_reload_activities_empty_clamps_index() {
        let activities = vec![ActivityConfig {
            name: "x".into(), details: "d".into(), state: "s".into(),
            large_image: None, large_text: None, small_image: None, small_text: None,
            duration_seconds: 5, buttons: None,
        }];
        let mut manager = DiscordPresenceManager::new("test_id", activities);
        manager.next_activity(); // index wraps to 0

        manager.reload_activities(vec![]);
        assert_eq!(manager.current_activity_index, 0);
    }

    #[test]
    fn test_get_status_empty_activities() {
        let manager = DiscordPresenceManager::new("test_id", vec![]);
        let status = manager.get_status();
        assert!(status.contains("Sin actividades"));
    }

    #[test]
    fn test_get_status_with_activities() {
        let activities = vec![ActivityConfig {
            name: "working".into(), details: "🎨 Creating".into(), state: "In the zone".into(),
            large_image: None, large_text: None, small_image: None, small_text: None,
            duration_seconds: 30, buttons: None,
        }];
        let manager = DiscordPresenceManager::new("test_id", activities);
        let status = manager.get_status();
        assert!(status.contains("working"));
        assert!(status.contains("🎨 Creating"));
    }

    #[test]
    fn test_default_duration_when_empty() {
        let manager = DiscordPresenceManager::new("test_id", vec![]);
        assert_eq!(manager.get_current_activity_duration(), 30);
    }

    // --- Config tests ---

    #[test]
    fn test_default_config_is_valid() {
        let config = get_default_config();
        assert_eq!(config.discord.app_id, "1234567890123456789");
        assert!(!config.discord.activities.is_empty());
    }

    #[test]
    fn test_default_config_serializes() {
        let config = get_default_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("app_id"));
        assert!(json.contains("activities"));

        // Verify it can be deserialized back
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.discord.app_id, config.discord.app_id);
        assert_eq!(deserialized.discord.activities.len(), config.discord.activities.len());
    }

    #[test]
    fn test_config_deserializes_correctly() {
        let json = r#"{
            "discord": {
                "app_id": "12345",
                "activities": [
                    {
                        "name": "test",
                        "details": "Testing",
                        "state": "OK",
                        "duration_seconds": 10
                    }
                ]
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.discord.app_id, "12345");
        assert_eq!(config.discord.activities.len(), 1);
        assert_eq!(config.discord.activities[0].name, "test");
        assert_eq!(config.discord.activities[0].details, "Testing");
        assert_eq!(config.discord.activities[0].duration_seconds, 10);
        assert!(config.discord.activities[0].large_image.is_none());
    }

    #[test]
    fn test_config_with_buttons_deserializes() {
        let json = r#"{
            "discord": {
                "app_id": "test_id",
                "activities": [
                    {
                        "name": "with_buttons",
                        "details": "Has buttons",
                        "state": "active",
                        "duration_seconds": 30,
                        "buttons": [
                            {"label": "Visit", "url": "https://example.com"},
                            {"label": "Contact", "url": "https://example.com/contact"}
                        ]
                    }
                ]
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        let activity = &config.discord.activities[0];
        assert!(activity.buttons.is_some());
        let buttons = activity.buttons.as_ref().unwrap();
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0].label, "Visit");
        assert_eq!(buttons[1].url, "https://example.com/contact");
    }

    #[test]
    fn test_config_with_images_deserializes() {
        let json = r#"{
            "discord": {
                "app_id": "test_id",
                "activities": [
                    {
                        "name": "with_images",
                        "details": "Images test",
                        "state": "active",
                        "duration_seconds": 30,
                        "large_image": "main_logo",
                        "large_text": "Main Logo",
                        "small_image": "icon",
                        "small_text": "Icon"
                    }
                ]
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        let activity = &config.discord.activities[0];
        assert_eq!(activity.large_image.as_deref(), Some("main_logo"));
        assert_eq!(activity.large_text.as_deref(), Some("Main Logo"));
        assert_eq!(activity.small_image.as_deref(), Some("icon"));
        assert_eq!(activity.small_text.as_deref(), Some("Icon"));
    }

    #[test]
    fn test_invalid_config_errors() {
        let bad_json = r#"{"discord": {"app_id": 123}}"#;
        let result: std::result::Result<Config, _> = serde_json::from_str(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_app_data_dir_returns_something() {
        let dir = get_app_data_dir().unwrap();
        assert!(!dir.as_os_str().is_empty());
    }

    // --- ButtonConfig tests ---

    #[test]
    fn test_button_config_serialization_roundtrip() {
        let button = ButtonConfig {
            label: "Test".into(),
            url: "https://test.com".into(),
        };
        let json = serde_json::to_string(&button).unwrap();
        let deserialized: ButtonConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "Test");
        assert_eq!(deserialized.url, "https://test.com");
    }
}
