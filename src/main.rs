#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::{Path, PathBuf};
use std::io::Write;
use std::env;

#[cfg(windows)]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::*,
        UI::{Shell::*, WindowsAndMessaging::*},
        Graphics::Gdi::*,
        System::{LibraryLoader::*, Console::*},
    },
};

// Alias para evitar conflictos con windows::core::Result
type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug, Deserialize, Serialize)]
struct ButtonConfig {
    label: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ActivityConfig {
    name: String,
    details: String,
    state: String,
    large_image: Option<String>,
    large_text: Option<String>,
    small_image: Option<String>,
    small_text: Option<String>,
    duration_seconds: u64,
    buttons: Option<Vec<ButtonConfig>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DiscordConfig {
    app_id: String,
    activities: Vec<ActivityConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    discord: DiscordConfig,
}

#[derive(Debug, Clone)]
enum AppCommand {
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
struct AppState {
    is_paused: bool,
    is_running: bool,
}

#[derive(Debug)]
struct DiscordPresenceManager {
    client: Option<DiscordIpcClient>,
    app_id: String,
    activities: Vec<ActivityConfig>,
    current_activity_index: usize,
    is_connected: bool,
    last_connection_attempt: SystemTime,
}

impl DiscordPresenceManager {
    fn new(app_id: &str, activities: Vec<ActivityConfig>) -> Self {
        Self {
            client: None,
            app_id: app_id.to_string(),
            activities,
            current_activity_index: 0,
            is_connected: false,
            last_connection_attempt: SystemTime::UNIX_EPOCH,
        }
    }

    async fn connect(&mut self) -> StdResult<(), String> {
        self.last_connection_attempt = SystemTime::now();
        
        let mut client = DiscordIpcClient::new(&self.app_id)
            .map_err(|e| format!("Error creando cliente Discord: {}", e))?;
        client.connect()
            .map_err(|e| format!("Error conectando a Discord: {}", e))?;
        
        log_info("Discord RPC conectado exitosamente!");
        self.client = Some(client);
        self.is_connected = true;
        Ok(())
    }

    async fn try_reconnect(&mut self) -> bool {
        // No intentar reconectar más de una vez cada 5 segundos
        if let Ok(elapsed) = self.last_connection_attempt.elapsed() {
            if elapsed.as_secs() < 5 {
                return false;
            }
        }

        log_info("🔄 Intentando reconectar a Discord...");
        match self.connect().await {
            Ok(_) => {
                log_info("✅ Reconexión exitosa!");
                // Intentar restaurar la actividad actual
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

    fn is_connection_alive(&self) -> bool {
        self.is_connected && self.client.is_some()
    }

    async fn set_current_activity(&mut self) -> StdResult<(), String> {
        if let Some(client) = &mut self.client {
            if self.activities.is_empty() {
                return Ok(());
            }

            let activity_config = &self.activities[self.current_activity_index];
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut activity = discord_rich_presence::activity::Activity::new()
                .details(&activity_config.details)
                .state(&activity_config.state)
                .timestamps(
                    discord_rich_presence::activity::Timestamps::new().start(timestamp as i64),
                );

            // Agregar imágenes si están configuradas
            if let Some(large_image) = &activity_config.large_image {
                let mut assets = discord_rich_presence::activity::Assets::new();
                assets = assets.large_image(large_image);
                
                if let Some(large_text) = &activity_config.large_text {
                    assets = assets.large_text(large_text);
                }
                
                if let Some(small_image) = &activity_config.small_image {
                    assets = assets.small_image(small_image);
                    
                    if let Some(small_text) = &activity_config.small_text {
                        assets = assets.small_text(small_text);
                    }
                }
                
                activity = activity.assets(assets);
            }

            // Agregar botones si están configurados
            if let Some(buttons) = &activity_config.buttons {
                let discord_buttons: Vec<discord_rich_presence::activity::Button> = buttons
                    .iter()
                    .map(|btn| discord_rich_presence::activity::Button::new(&btn.label, &btn.url))
                    .collect();
                
                activity = activity.buttons(discord_buttons);
            }

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

    fn get_current_activity_duration(&self) -> u64 {
        if self.activities.is_empty() {
            return 30; // valor por defecto
        }
        self.activities[self.current_activity_index].duration_seconds
    }

    fn next_activity(&mut self) {
        if !self.activities.is_empty() {
            self.current_activity_index = (self.current_activity_index + 1) % self.activities.len();
        }
    }

    fn reload_activities(&mut self, new_activities: Vec<ActivityConfig>) {
        self.activities = new_activities;
        if self.current_activity_index >= self.activities.len() {
            self.current_activity_index = 0;
        }
        log_info(&format!("🔄 Actividades recargadas: {} disponibles", self.activities.len()));
    }

    fn get_status(&self) -> String {
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

    async fn clear_activity(&mut self) -> StdResult<(), String> {
        if let Some(client) = &mut self.client {
            client.clear_activity()
                .map_err(|e| format!("Error limpiando actividad: {}", e))?;
            log_info("Actividad de Discord limpiada");
        }
        Ok(())
    }

    async fn disconnect(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
            println!("Discord RPC desconectado");
        }
        self.is_connected = false;
        self.client = None;
    }
}

fn get_app_data_dir() -> StdResult<PathBuf, String> {
    #[cfg(windows)]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            let app_dir = PathBuf::from(appdata).join("lorianworkspace");
            return Ok(app_dir);
        }
    }
    
    // Fallback para otros sistemas o si APPDATA no existe
    if let Ok(home) = env::var("HOME") {
        let app_dir = PathBuf::from(home).join(".lorianworkspace");
        return Ok(app_dir);
    }
    
    // Último fallback - directorio actual
    Ok(PathBuf::from("."))
}

fn ensure_app_data_dir_exists() -> StdResult<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Error creando directorio {}: {}", app_dir.display(), e))?;
        println!("📁 Creado directorio: {}", app_dir.display());
    }
    
    Ok(app_dir)
}

fn get_default_config() -> Config {
    Config {
        discord: DiscordConfig {
            app_id: "1234567890123456789".to_string(), // ⚠️ CAMBIAR ESTE ID
            activities: vec![
                ActivityConfig {
                    name: "commission".to_string(),
                    details: "💼 Open for Commissions".to_string(),
                    state: "Taking art requests".to_string(),
                    large_image: Some("award".to_string()),
                    large_text: Some("Commission Status".to_string()),
                    small_image: Some("thanks".to_string()),
                    small_text: Some("Available now".to_string()),
                    duration_seconds: 3,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "💰 Commission Info".to_string(),
                            url: "https://example.com/commissions".to_string(),
                        },
                        ButtonConfig {
                            label: "🎨 Portfolio".to_string(), 
                            url: "https://example.com/portfolio".to_string(),
                        }
                    ]),
                },
                ActivityConfig {
                    name: "working".to_string(),
                    details: "🎨 Working on Commission".to_string(),
                    state: "Creating digital art".to_string(),
                    large_image: Some("mostrando_1".to_string()),
                    large_text: Some("Work in Progress".to_string()),
                    small_image: Some("talk_1".to_string()),
                    small_text: Some("Focused".to_string()),
                    duration_seconds: 5,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "📞 Contact Me".to_string(),
                            url: "https://example.com/contact".to_string(),
                        }
                    ]),
                },
                ActivityConfig {
                    name: "chatting".to_string(),
                    details: "💬 Chatting with Clients".to_string(),
                    state: "Discussing projects".to_string(),
                    large_image: Some("talk_2".to_string()),
                    large_text: Some("Client Communication".to_string()),
                    small_image: Some("hmmm".to_string()),
                    small_text: Some("Thinking...".to_string()),
                    duration_seconds: 4,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "💬 Chat with Me".to_string(),
                            url: "https://example.com/chat".to_string(),
                        },
                        ButtonConfig {
                            label: "📋 My Services".to_string(),
                            url: "https://example.com/services".to_string(),
                        }
                    ]),
                },
                ActivityConfig {
                    name: "surprised".to_string(),
                    details: "😲 Amazing Results!".to_string(),
                    state: "Just finished a piece".to_string(),
                    large_image: Some("wow".to_string()),
                    large_text: Some("Wow!".to_string()),
                    small_image: Some("thanks".to_string()),
                    small_text: Some("Grateful".to_string()),
                    duration_seconds: 2,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "🖼️ See Latest Work".to_string(),
                            url: "https://example.com/latest".to_string(),
                        }
                    ]),
                },
                ActivityConfig {
                    name: "confused".to_string(),
                    details: "🤔 Figuring things out".to_string(),
                    state: "Planning next project".to_string(),
                    large_image: Some("what".to_string()),
                    large_text: Some("What to do next?".to_string()),
                    small_image: Some("talk_3".to_string()),
                    small_text: Some("Brainstorming".to_string()),
                    duration_seconds: 3,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "💡 Suggest Ideas".to_string(),
                            url: "https://example.com/suggestions".to_string(),
                        }
                    ]),
                },
                ActivityConfig {
                    name: "cute_mode".to_string(),
                    details: "🐱 Nya~ Mode Active".to_string(),
                    state: "Being adorable".to_string(),
                    large_image: Some("nya".to_string()),
                    large_text: Some("Nya nya~".to_string()),
                    small_image: Some("talk_5".to_string()),
                    small_text: Some("Kawaii".to_string()),
                    duration_seconds: 6,
                    buttons: Some(vec![
                        ButtonConfig {
                            label: "🐾 Pet the Cat".to_string(),
                            url: "https://example.com/nya".to_string(),
                        },
                        ButtonConfig {
                            label: "😸 More Cute Stuff".to_string(),
                            url: "https://example.com/cute".to_string(),
                        }
                    ]),
                }
            ],
        }
    }
}

fn create_default_config_file(config_path: &Path) -> StdResult<(), String> {
    let default_config = get_default_config();
    let config_json = serde_json::to_string_pretty(&default_config)
        .map_err(|e| format!("Error serializando configuración default: {}", e))?;
    
    fs::write(config_path, config_json)
        .map_err(|e| format!("Error escribiendo archivo de configuración: {}", e))?;
    
    log_info(&format!("📝 Creado archivo de configuración default: {}", config_path.display()));
    Ok(())
}

fn load_config() -> StdResult<Config, String> {
    let app_dir = ensure_app_data_dir_exists()?;
    let config_path = app_dir.join("config.json");
    
    // Si no existe el config, crear uno por defecto
    if !config_path.exists() {
        log_info("🆕 Primera ejecución - creando configuración default...");
        create_default_config_file(&config_path)?;
        log_info(&format!("📍 Configuración guardada en: {}", config_path.display()));
        log_info("💡 Puedes editar este archivo para personalizar tu app");
    }
    
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Error leyendo {}: {}", config_path.display(), e))?;
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| format!("Error parseando JSON en {}: {}", config_path.display(), e))?;
    
    Ok(config)
}

fn setup_file_watcher(command_sender: Sender<AppCommand>) -> StdResult<RecommendedWatcher, notify::Error> {
    let mut watcher = RecommendedWatcher::new(
        move |res: StdResult<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Solo procesar eventos de modificación de archivos
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

    // Vigilar el directorio de AppData para cambios en config.json
    match get_app_data_dir() {
        Ok(app_dir) => {
            watcher.watch(&app_dir, RecursiveMode::NonRecursive)?;
            log_info(&format!("👁️  File watcher iniciado para: {}", app_dir.join("config.json").display()));
        }
        Err(e) => {
            log_error(&format!("⚠️  Error obteniendo directorio de AppData: {}", e));
            // Fallback al directorio actual
            watcher.watch(Path::new("."), RecursiveMode::NonRecursive)?;
            log_info("👁️  File watcher iniciado en directorio actual");
        }
    }
    
    Ok(watcher)
}

// Las funciones de terminal menu fueron removidas ya que la app funciona sin consola

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

    // Variable global para el command sender (necesaria para window_proc)
    static mut GLOBAL_COMMAND_SENDER: Option<Sender<AppCommand>> = None;
    static mut GLOBAL_APP_STATE: Option<Arc<Mutex<AppState>>> = None;

    pub struct SystemTray {
        hwnd: HWND,
        command_sender: Sender<AppCommand>,
    }

    impl SystemTray {
        pub fn new(command_sender: Sender<AppCommand>) -> windows::core::Result<Self> {
            unsafe {
                // Almacenar el command_sender globalmente para usar en window_proc
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
                    lpszClassName: w!("DiscordBgApp"),
                    hIconSm: HICON(std::ptr::null_mut()),
                };

                RegisterClassExW(&window_class);

                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("DiscordBgApp"),
                    w!("Lorian Workspace"),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    None,
                    None,
                    instance,
                    None,
                );

                let hwnd = hwnd?;

                let tray = Self {
                    hwnd,
                    command_sender,
                };

                tray.add_tray_icon()?;
                Ok(tray)
            }
        }

        unsafe fn add_tray_icon(&self) -> windows::core::Result<()> {
            log_info("Intentando crear icono del system tray...");
            
            // Intentar cargar icono personalizado del ejecutable, o usar uno por defecto
            let icon = if let Ok(module) = GetModuleHandleW(None) {
                // Intentar cargar icono ID 1 del ejecutable
                LoadIconW(module, PCWSTR(1 as *const u16))
                    .unwrap_or_else(|_| {
                        // Fallback a icono sistema si no se encuentra el personalizado
                        LoadIconW(None, IDI_INFORMATION).unwrap_or_default()
                    })
            } else {
                LoadIconW(None, IDI_INFORMATION).unwrap_or_default()
            };
            
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

            let tip = "Lorian Workspace - Doble click: mostrar consola";
            let tip_wide: Vec<u16> = tip.encode_utf16().collect();
            let copy_len = tip_wide.len().min(127);
            nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            if result.as_bool() {
                log_info("✅ Icono del system tray creado exitosamente");
            } else {
                log_error("❌ Fallo al crear icono del system tray");
            }
            
            Ok(())
        }

        unsafe fn show_context_menu(&self, is_paused: bool) {
            let hmenu = CreatePopupMenu().unwrap();
            
            // Control de reproducción
            if is_paused {
                AppendMenuW(hmenu, MF_STRING, ID_MENU_RESUME as usize, w!("▶️ Reanudar"));
            } else {
                AppendMenuW(hmenu, MF_STRING, ID_MENU_PAUSE as usize, w!("⏸️ Pausar"));
            }
            AppendMenuW(hmenu, MF_STRING, ID_MENU_NEXT as usize, w!("⏭️ Siguiente Actividad"));
            
            AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
            
            // Configuración y utilidades
            AppendMenuW(hmenu, MF_STRING, ID_MENU_TOGGLE_CONSOLE as usize, w!("💻 Mostrar/Ocultar Consola"));
            AppendMenuW(hmenu, MF_STRING, ID_MENU_OPEN_CONFIG as usize, w!("📝 Abrir Configuración"));
            AppendMenuW(hmenu, MF_STRING, ID_MENU_RELOAD as usize, w!("🔄 Recargar Config"));
            AppendMenuW(hmenu, MF_STRING, ID_MENU_STATUS as usize, w!("📊 Ver Estado"));
            
            AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
            AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT as usize, w!("❌ Salir"));

            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);

            SetForegroundWindow(self.hwnd);
            
            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RIGHTBUTTON | TPM_RETURNCMD,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );

            if cmd.as_bool() {
                self.handle_menu_command(cmd.0 as u32);
            }

            DestroyMenu(hmenu);
        }

        fn handle_menu_command(&self, cmd: u32) {
            let command = match cmd {
                ID_MENU_PAUSE => AppCommand::Pause,
                ID_MENU_RESUME => AppCommand::Resume,
                ID_MENU_NEXT => AppCommand::NextActivity,
                ID_MENU_RELOAD => AppCommand::ReloadConfig,
                ID_MENU_STATUS => AppCommand::ShowStatus,
                ID_MENU_TOGGLE_CONSOLE => AppCommand::ToggleConsole,
                ID_MENU_OPEN_CONFIG => AppCommand::OpenConfig,
                ID_MENU_EXIT => AppCommand::Exit,
                _ => return,
            };

            let _ = self.command_sender.send(command);
        }

        fn handle_global_menu_command(cmd: u32) {
            let command = match cmd {
                ID_MENU_PAUSE => AppCommand::Pause,
                ID_MENU_RESUME => AppCommand::Resume,
                ID_MENU_NEXT => AppCommand::NextActivity,
                ID_MENU_RELOAD => AppCommand::ReloadConfig,
                ID_MENU_STATUS => AppCommand::ShowStatus,
                ID_MENU_TOGGLE_CONSOLE => AppCommand::ToggleConsole,
                ID_MENU_OPEN_CONFIG => AppCommand::OpenConfig,
                ID_MENU_EXIT => AppCommand::Exit,
                _ => return,
            };

            unsafe {
                if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                    let _ = sender.send(command);
                }
            }
        }

        unsafe extern "system" fn window_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            match msg {
                WM_TRAYICON => {
                    let event = loword(lparam.0 as u32) as u32;
                    log_info(&format!("Evento del tray recibido en window_proc: {}", event));
                    
                    match event {
                        WM_RBUTTONUP => {
                            log_info("Click derecho en tray - enviando comando para mostrar menú");
                            // Click derecho - mostrar menú contextual
                            if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                                let _ = sender.send(AppCommand::ShowStatus);
                                
                                // Crear menú contextual inmediatamente
                                let hmenu = CreatePopupMenu().unwrap();
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_PAUSE as usize, w!("⏸️ Pausar/Reanudar"));
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_NEXT as usize, w!("⏭️ Siguiente Actividad"));
                                AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_TOGGLE_CONSOLE as usize, w!("💻 Mostrar Consola"));
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_OPEN_CONFIG as usize, w!("📝 Abrir Config"));
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_RELOAD as usize, w!("🔄 Recargar"));
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_STATUS as usize, w!("📊 Estado"));
                                AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                                AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT as usize, w!("❌ Salir"));

                                let mut pt = POINT { x: 0, y: 0 };
                                GetCursorPos(&mut pt);
                                SetForegroundWindow(hwnd);
                                
                                let cmd = TrackPopupMenu(
                                    hmenu,
                                    TPM_RIGHTBUTTON | TPM_RETURNCMD,
                                    pt.x,
                                    pt.y,
                                    0,
                                    hwnd,
                                    None,
                                );

                                if cmd.as_bool() {
                                    Self::handle_global_menu_command(cmd.0 as u32);
                                }
                                DestroyMenu(hmenu);
                            }
                        }
                        WM_LBUTTONDBLCLK => {
                            log_info("Doble click izquierdo en tray - alternando consola");
                            // Doble click izquierdo - toggle consola
                            if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                                let _ = sender.send(AppCommand::ToggleConsole);
                            }
                        }
                        WM_LBUTTONUP => {
                            log_info("Click izquierdo simple en tray - mostrando estado");
                            // Click izquierdo simple - mostrar estado
                            if let Some(sender) = &GLOBAL_COMMAND_SENDER {
                                let _ = sender.send(AppCommand::ShowStatus);
                            }
                        }
                        _ => {
                            log_info(&format!("Evento del tray no manejado: {}", event));
                        }
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    log_info("WM_DESTROY recibido - cerrando aplicación");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }

        pub fn process_messages(&self, _app_state: Arc<Mutex<AppState>>) -> bool {
            unsafe {
                let mut msg = MSG::default();
                // PeekMessage en lugar de GetMessage para non-blocking
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        log_info("WM_QUIT recibido en process_messages");
                        return false; // Señal para terminar
                    }

                    // Los eventos del tray se manejan directamente en window_proc
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
        pub fn new(_: Sender<AppCommand>) -> Result<Self, Box<dyn std::error::Error>> {
            println!("⚠️  System tray no disponible en esta plataforma");
            Ok(SystemTray)
        }
        
        pub fn run_message_loop(&self, _: Arc<Mutex<AppState>>) {
            // No-op para plataformas no Windows
        }
    }
}

// Función simplificada que no depende de tray-icon por ahora
fn setup_app() -> StdResult<(), String> {
    log_info("⚙️  Configurando aplicación...");
    log_info("📝 Aplicación configurada para funcionar en background sin consola");
    #[cfg(windows)]
    log_info("🖥️  Detectado: Windows - Implementación del tray habilitada");
    #[cfg(not(windows))]
    log_info("🐧 Detectado: Linux - Usando implementación alternativa");
    Ok(())
}

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
    log_info("Intentando mostrar consola...");
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            log_info("Ventana de consola encontrada, mostrando...");
            let _ = ShowWindow(console_window, SW_SHOW);
            let _ = SetForegroundWindow(console_window);
        } else {
            log_info("No hay consola, creando una nueva...");
            // Si no hay consola, crear una nueva
            if AllocConsole().is_ok() {
                log_info("Consola creada, configurando...");
                let new_console = GetConsoleWindow();
                if !new_console.is_invalid() {
                    let _ = ShowWindow(new_console, SW_SHOW);
                    let _ = SetForegroundWindow(new_console);
                    log_info("✅ Nueva consola mostrada exitosamente");
                } else {
                    log_error("❌ Error obteniendo handle de nueva consola");
                }
            } else {
                log_error("❌ Error creando nueva consola");
            }
        }
    }
}

#[cfg(windows)]
fn toggle_console() {
    log_info("Alternando visibilidad de consola...");
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            if IsWindowVisible(console_window).as_bool() {
                log_info("Consola visible, ocultando...");
                let _ = ShowWindow(console_window, SW_HIDE);
            } else {
                log_info("Consola oculta, mostrando...");
                let _ = ShowWindow(console_window, SW_SHOW);
                let _ = SetForegroundWindow(console_window);
            }
        } else {
            log_info("No hay consola, creando una nueva...");
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
                let _ = ShellExecuteW(
                    None,
                    None,
                    windows::core::PCWSTR(path_wide.as_ptr()),
                    None,
                    None,
                    SW_SHOW
                );
            }
        }
    }
}

#[cfg(not(windows))]
fn hide_console() {}

#[cfg(not(windows))]
fn show_console() {}

#[cfg(not(windows))]
fn toggle_console() {}

#[cfg(not(windows))]
fn open_config_file() {}

// Sistema de logging
fn get_log_file_path() -> StdResult<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("app.log"))
}

fn log_message(message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Formatear timestamp de manera más legible
    let datetime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    let timestamp_str = format!("{:?}", datetime);
    
    let log_entry = format!("[{}] {}\n", timestamp_str, message);
    
    // Intentar escribir al archivo de log
    if let Ok(log_path) = get_log_file_path() {
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }
    
    // NO imprimir a stdout en subsystem windows para evitar problemas de consola
    #[cfg(not(windows))]
    {
    print!("{}", log_entry);
    let _ = io::stdout().flush();
    }
}

fn log_error(message: &str) {
    log_message(&format!("ERROR: {}", message));
}

fn log_info(message: &str) {
    log_message(&format!("INFO: {}", message));
}

#[tokio::main]
async fn main() -> StdResult<(), String> {
    log_info("🚀 Iniciando Lorian Workspace...");
    log_info("📦 Iniciando en modo background sin consola");
    log_info("💡 Usa el icono del system tray para controlar la app");
    
    // Como es una app de Windows sin consola, no necesitamos ocultar nada

    // Cargar configuración desde config.json
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log_error(&format!("❌ Error cargando config.json: {}", e));
            log_error("💡 Asegúrate de que el archivo config.json exista y tenga el formato correcto");
            return Err(e.into());
        }
    };

    if config.discord.activities.is_empty() {
        log_error("⚠️  No hay actividades configuradas en config.json");
        return Ok(());
    }

    log_info(&format!("📋 Cargadas {} actividades desde config.json", config.discord.activities.len()));
    for (i, activity) in config.discord.activities.iter().enumerate() {
        log_info(&format!("   {}. {} - {} segundos", i + 1, activity.name, activity.duration_seconds));
    }
    
    // Mostrar ubicación del archivo de configuración
    if let Ok(app_dir) = get_app_data_dir() {
        let config_path = app_dir.join("config.json");
        log_info(&format!("📁 Archivo de configuración: {}", config_path.display()));
        log_info("💡 Edita este archivo para personalizar tus actividades");
    }

    let discord_manager = Arc::new(Mutex::new(DiscordPresenceManager::new(
        &config.discord.app_id,
        config.discord.activities,
    )));

    let app_state = Arc::new(Mutex::new(AppState {
        is_paused: false,
        is_running: true,
    }));

    // Canal de comandos para el system tray
    let (command_sender, command_receiver): (Sender<AppCommand>, Receiver<AppCommand>) = unbounded();

    // Configurar aplicación
    setup_app()?;
    
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
                log_error("💡 Asegúrate de que Discord esté abierto y que tengas un Application ID válido");
                log_error("📋 Ve al setup_discord.md para instrucciones de configuración");
                log_error("🖼️  Verifica que hayas subido las imágenes al Discord Developer Portal");
            }
        }
    }

    // Inicializar System Tray
    log_info("Iniciando System Tray...");
    let _tray = tray::SystemTray::new(command_sender.clone())
        .map_err(|e| {
            log_error(&format!("Error iniciando system tray: {}", e));
            format!("Error iniciando system tray: {}", e)
        })?;
    log_info("🖱️  System Tray iniciado - busca el icono en la bandeja del sistema");
    log_info("💡 Click derecho en el icono para ver opciones");

    // Inicializar File Watcher para hot reload
    let _file_watcher = setup_file_watcher(command_sender.clone())
        .map_err(|e| format!("Error iniciando file watcher: {}", e))?;
    log_info("🔥 Hot reload activado - edita config.json y se recargará automáticamente");

    // Ya no necesitamos menú terminal para aplicación de bandeja
    log_info("💻 Aplicación configurada para control via tray icon");

    // Sistema de rotación de actividades con auto-reconexión
    let discord_manager_clone = discord_manager.clone();
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        loop {
            let (duration, is_paused, is_connected) = {
                let manager = discord_manager_clone.lock().await;
                let state = app_state_clone.lock().await;
                (manager.get_current_activity_duration(), state.is_paused, manager.is_connection_alive())
            };
            
            // Si no está conectado, intentar reconectar
            if !is_connected {
                let mut manager = discord_manager_clone.lock().await;
                if manager.try_reconnect().await {
                    log_info("🔗 Reconexión exitosa - continuando rotación");
                } else {
                    // Esperar antes de intentar nuevamente
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            }
            
            // Esperar la duración especificada para la actividad actual
            tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;
            
            // Solo cambiar actividad si no está pausado y está conectado
            if !is_paused {
                let mut manager = discord_manager_clone.lock().await;
                if manager.is_connection_alive() {
                    manager.next_activity();
                    if let Err(_) = manager.set_current_activity().await {
                        // Si falla al establecer actividad, marcar como desconectado
                        // El próximo ciclo intentará reconectar
                        log_error("💔 Perdida conexión con Discord - intentando reconectar...");
                    }
                }
            }
        }
    });

    // Manejo de comandos del tray
    let discord_manager_clone2 = discord_manager.clone();
    let app_state_clone2 = app_state.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(command) = command_receiver.recv() {
                match command {
                    AppCommand::Pause => {
                        let mut state = app_state_clone2.lock().await;
                        state.is_paused = !state.is_paused;
                        if state.is_paused {
                            log_info("⏸️  Actividades pausadas");
                        } else {
                            log_info("▶️ Actividades reanudadas");
                        }
                    }
                    AppCommand::Resume => {
                        let mut state = app_state_clone2.lock().await;
                        state.is_paused = false;
                        log_info("▶️ Actividades reanudadas");
                    }
                    AppCommand::NextActivity => {
                        let mut manager = discord_manager_clone2.lock().await;
                        if !manager.is_connection_alive() {
                            if manager.try_reconnect().await {
                                log_info("🔗 Reconectado antes de cambiar actividad");
                            } else {
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
                        // Pequeño delay para evitar múltiples recargas rápidas
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        
                        let config_result = load_config();
                        match config_result {
                            Ok(new_config) => {
                                let mut manager = discord_manager_clone2.lock().await;
                                let old_count = manager.activities.len();
                                manager.reload_activities(new_config.discord.activities);
                                let new_count = manager.activities.len();
                                
                                if let Err(e) = manager.set_current_activity().await {
                                    log_error(&format!("⚠️  Error aplicando nueva configuración: {}", e));
                                } else {
                                    log_info(&format!("🔄 Configuración recargada: {} → {} actividades", old_count, new_count));
                                    log_info("✨ Cambios aplicados automáticamente");
                                }
                            }
                            Err(e) => {
                                log_error(&format!("❌ Error recargando configuración: {}", e));
                                log_error("💡 Verifica la sintaxis del JSON en config.json");
                            }
                        }
                    }
                    AppCommand::ShowStatus => {
                        let manager = discord_manager_clone2.lock().await;
                        let state = app_state_clone2.lock().await;
                        log_info("📊 === Estado de la Aplicación ===");
                        log_info(&manager.get_status());
                        log_info(&format!("⏸️  Rotación: {}", if state.is_paused { "Pausada" } else { "Activa" }));
                        log_info(&format!("🔗 Conexión: {}", if manager.is_connection_alive() { "✅ Conectado" } else { "❌ Desconectado" }));
                        log_info(&format!("📱 App: {}", if state.is_running { "🟢 Funcionando" } else { "🔴 Cerrando" }));
                        
                        // Mostrar ubicación del config
                        if let Ok(app_dir) = get_app_data_dir() {
                            let config_path = app_dir.join("config.json");
                            log_info(&format!("📁 Config: {}", config_path.display()));
                        }
                    }
                    AppCommand::ToggleConsole => {
                        log_info("💻 Alternando visibilidad de consola...");
                        toggle_console();
                    }
                    AppCommand::OpenConfig => {
                        log_info("📝 Abriendo archivo de configuración...");
                        open_config_file();
                    }
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

    // Ejecutar el message loop del tray en un hilo separado
    // Message loop del tray se integrará con el main loop

    // Loop principal - esperar hasta que se solicite salir
    loop {
        // Procesar mensajes del tray (solo en Windows)
        #[cfg(windows)]
        {
            if !_tray.process_messages(app_state.clone()) {
                break; // WM_QUIT recibido
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
