//! Data model for the rebuilt X11 control center.
//!
//! The numbers and rules in this module come from the original program and its
//! resource files.  The evidence and the function addresses are listed in
//! `MECHANICS.md` and mirrored in `mechanics.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const APP_VERSION: &str = "1.0.0";
/// Physical buttons shown in the rebuild (hardware ids 1, 2, 3, 7 and 8).
pub const BUTTON_COUNT: usize = 5;
/// The original page and its command map have 18 assignable slots.
pub const BUTTON_SLOT_COUNT: usize = 18;
/// Eight stage slots exist; the page shows six and hides slots 7 and 8.
pub const DPI_STAGE_COUNT: usize = 8;
/// The factory active stage is the second slot (`FUN_004a4670`, offset 0x934).
pub const DPI_ACTIVE_DEFAULT: usize = 1;
pub const DPI_MIN: u32 = 50;
pub const DPI_MAX: u32 = 22_000;
pub const DPI_SPLIT: u32 = 10_000;
pub const DPI_POSITION_MAX: u32 = 320;
pub const POLLING_RATES: [u32; 4] = [125, 250, 500, 1_000];
/// Raw period byte the mouse reports for each index: 8 ms, 4 ms, 2 ms, 1 ms.
pub const POLLING_REPORT_CODES: [u8; 4] = [8, 4, 2, 1];
/// `FUN_0049e7a0` clamps the lighting mode to 0..=6 (seven real modes).
pub const LIGHT_MODE_COUNT: usize = 7;
/// Factory DPI slider positions from `FUN_004a4670`.
pub const DEFAULT_DPI_POSITIONS: [u32; DPI_STAGE_COUNT] = [16, 32, 48, 64, 100, 320, 0, 0];
/// Factory per-stage colours from `FUN_004a4670` (0x0000FF, 0x00FF00, ...).
pub const DEFAULT_STAGE_COLORS: [[u8; 3]; DPI_STAGE_COUNT] = [
    [0x00, 0x00, 0xFF],
    [0x00, 0xFF, 0x00],
    [0xFF, 0x00, 0x00],
    [0x00, 0xFF, 0xFF],
    [0xFF, 0xFF, 0x00],
    [0xFF, 0x00, 0xFF],
    [0x00, 0x40, 0xFF],
    [0xFF, 0xFF, 0xFF],
];
/// Factory light colour, `FUN_004a4670` writes 0x00FF00 at 0x95c.
pub const DEFAULT_LIGHT_COLOR: [u8; 3] = [0x00, 0xFF, 0x00];
/// Factory button bytes, `FUN_004a4670` copies them from DAT_004ec8f0.
pub const DEFAULT_BUTTON_CODES: [u8; BUTTON_SLOT_COUNT] = [
    0x01, 0x02, 0x03, 0x00, 0x00, 0x06, 0x04, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0B, 0x0C,
];
/// The original sets three repeating battery/status timers at 6000 ms.
pub const BATTERY_POLL_MS: u32 = 6_000;
/// `FUN_004a04f0` restarts timer 0x231 at 500 ms after a slider move.
pub const SLIDER_APPLY_MS: u32 = 500;
/// `FUN_004a01c0` restarts timer 0x231 at 200 ms after a combo change.
pub const COMBO_APPLY_MS: u32 = 200;
pub const COLOR_PRESETS: [[u8; 3]; 16] = [
    [0x00, 0xFF, 0xFF],
    [0x6C, 0xFF, 0x00],
    [0x7F, 0x00, 0xFF],
    [0x00, 0x7F, 0xFF],
    [0x00, 0x00, 0xFF],
    [0x7F, 0xFF, 0x00],
    [0xFF, 0x54, 0x00],
    [0xFF, 0x7F, 0x00],
    [0xFF, 0xB4, 0x00],
    [0xFF, 0xFF, 0x00],
    [0xFF, 0x00, 0x00],
    [0xFF, 0x00, 0xFF],
    [0x1E, 0xB4, 0x0A],
    [0x48, 0xB1, 0xFF],
    [0xFF, 0x48, 0xB0],
    [0xFF, 0xFF, 0xFF],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lang {
    Es,
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Self::Es
    }
}

impl Lang {
    pub fn t(self, es: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Es => es,
            Self::En => en,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Es => "Español",
            Self::En => "English",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Accent {
    Coral,
    Cyan,
    Violet,
    Mint,
}

impl Default for Accent {
    fn default() -> Self {
        Self::Coral
    }
}

impl Accent {
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Coral => (0xFF, 0x5C, 0x4D),
            Self::Cyan => (0x33, 0xD6, 0xF5),
            Self::Violet => (0xA7, 0x8B, 0xFA),
            Self::Mint => (0x4A, 0xDE, 0x80),
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Coral => lang.t("Coral", "Coral"),
            Self::Cyan => lang.t("Cian", "Cyan"),
            Self::Violet => lang.t("Violeta", "Violet"),
            Self::Mint => lang.t("Menta", "Mint"),
        }
    }

    pub const ALL: [Accent; 4] = [Accent::Coral, Accent::Cyan, Accent::Violet, Accent::Mint];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroMode {
    Repeat,
    HoldRelease,
    UntilKey,
}

impl Default for MacroMode {
    fn default() -> Self {
        Self::Repeat
    }
}

impl MacroMode {
    pub fn label(&self, lang: Lang) -> &'static str {
        match self {
            Self::Repeat => lang.t("Repetir N veces", "Repeat N times"),
            Self::HoldRelease => lang.t("Mantener pulsado", "Hold and release"),
            Self::UntilKey => lang.t("Hasta pulsar tecla", "Until key press"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    LeftClick,
    RightClick,
    MiddleClick,
    Forward,
    Backward,
    DoubleClick,
    FireButton,
    EasyAim,
    LedLoop,
    Disabled,
    DpiCycle,
    DpiUp,
    DpiDown,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    MediaPlayer,
    PlayPause,
    MediaStop,
    PrevTrack,
    NextTrack,
    VolumeUp,
    VolumeDown,
    Mute,
    BrowserHome,
    BrowserFavorites,
    BrowserForward,
    BrowserBackward,
    BrowserStop,
    BrowserRefresh,
    BrowserSearch,
    Email,
    Calculator,
    MyComputer,
    Cut,
    Copy,
    Paste,
    Open,
    Save,
    Find,
    Redo,
    SelectAll,
    Print,
    CloseWindow,
    SwapWindows,
    ShowDesktop,
    RunCommand,
    LockPc,
    ScreenCapture,
    Shortcut {
        ctrl: bool,
        shift: bool,
        alt: bool,
        win: bool,
        key: String,
    },
    Macro {
        name: String,
        mode: MacroMode,
        loops: u8,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Disabled
    }
}

impl Command {
    pub fn label(&self, lang: Lang) -> String {
        match self {
            Self::LeftClick => lang.t("Clic izquierdo", "Left click").into(),
            Self::RightClick => lang.t("Clic derecho", "Right click").into(),
            Self::MiddleClick => lang.t("Clic central", "Middle click").into(),
            Self::Forward => lang.t("Adelante", "Forward").into(),
            Self::Backward => lang.t("Atrás", "Backward").into(),
            Self::DoubleClick => lang.t("Doble clic", "Double click").into(),
            Self::FireButton => lang.t("Botón de fuego", "Fire button").into(),
            Self::EasyAim => lang.t("Puntería fácil", "Easy aim").into(),
            Self::LedLoop => lang.t("Ciclo de iluminación", "LED cycle").into(),
            Self::Disabled => lang.t("Desactivado", "Disabled").into(),
            Self::DpiCycle => lang.t("Ciclo de DPI", "DPI cycle").into(),
            Self::DpiUp => "DPI +".into(),
            Self::DpiDown => "DPI −".into(),
            Self::ScrollUp => lang.t("Desplazar arriba", "Scroll up").into(),
            Self::ScrollDown => lang.t("Desplazar abajo", "Scroll down").into(),
            Self::ScrollLeft => lang.t("Desplazar a la izquierda", "Scroll left").into(),
            Self::ScrollRight => lang.t("Desplazar a la derecha", "Scroll right").into(),
            Self::MediaPlayer => lang.t("Reproductor multimedia", "Media player").into(),
            Self::PlayPause => lang.t("Reproducir / pausa", "Play / pause").into(),
            Self::MediaStop => lang.t("Detener música", "Stop media").into(),
            Self::PrevTrack => lang.t("Pista anterior", "Previous track").into(),
            Self::NextTrack => lang.t("Pista siguiente", "Next track").into(),
            Self::VolumeUp => lang.t("Subir volumen", "Volume up").into(),
            Self::VolumeDown => lang.t("Bajar volumen", "Volume down").into(),
            Self::Mute => lang.t("Silenciar", "Mute").into(),
            Self::BrowserHome => lang.t("Inicio del navegador", "Browser home").into(),
            Self::BrowserFavorites => lang.t("Favoritos", "Favorites").into(),
            Self::BrowserForward => lang.t("Avanzar en el navegador", "Browser forward").into(),
            Self::BrowserBackward => lang.t("Volver en el navegador", "Browser back").into(),
            Self::BrowserStop => lang.t("Detener carga", "Browser stop").into(),
            Self::BrowserRefresh => lang.t("Actualizar página", "Refresh page").into(),
            Self::BrowserSearch => lang.t("Buscar en el navegador", "Browser search").into(),
            Self::Email => lang.t("Correo", "Email").into(),
            Self::Calculator => lang.t("Calculadora", "Calculator").into(),
            Self::MyComputer => lang.t("Mi equipo", "My computer").into(),
            Self::Cut => lang.t("Cortar", "Cut").into(),
            Self::Copy => lang.t("Copiar", "Copy").into(),
            Self::Paste => lang.t("Pegar", "Paste").into(),
            Self::Open => lang.t("Abrir", "Open").into(),
            Self::Save => lang.t("Guardar", "Save").into(),
            Self::Find => lang.t("Buscar", "Find").into(),
            Self::Redo => lang.t("Rehacer", "Redo").into(),
            Self::SelectAll => lang.t("Seleccionar todo", "Select all").into(),
            Self::Print => lang.t("Imprimir", "Print").into(),
            Self::CloseWindow => lang.t("Cerrar ventana", "Close window").into(),
            Self::SwapWindows => lang.t("Cambiar de ventana", "Swap windows").into(),
            Self::ShowDesktop => lang.t("Mostrar escritorio", "Show desktop").into(),
            Self::RunCommand => lang.t("Ejecutar comando", "Run command").into(),
            Self::LockPc => lang.t("Bloquear el equipo", "Lock PC").into(),
            Self::ScreenCapture => lang.t("Captura de pantalla", "Screen capture").into(),
            Self::Shortcut {
                ctrl,
                shift,
                alt,
                win,
                key,
            } => {
                let mut parts = Vec::new();
                if *ctrl {
                    parts.push("Ctrl");
                }
                if *shift {
                    parts.push("Shift");
                }
                if *alt {
                    parts.push("Alt");
                }
                if *win {
                    parts.push("Win");
                }
                parts.push(key);
                format!("{} {}", lang.t("Atajo:", "Shortcut:"), parts.join("+"))
            }
            Self::Macro { name, mode, loops } => {
                format!(
                    "{} {} · {} · {}×",
                    lang.t("Macro", "Macro"),
                    name,
                    mode.label(lang),
                    loops
                )
            }
        }
    }

    pub fn short_label(&self, lang: Lang) -> String {
        match self {
            Self::Shortcut { key, .. } => format!("⌨ {key}"),
            Self::Macro { name, .. } => format!("▶ {name}"),
            _ => self.label(lang),
        }
    }

    pub fn category(&self) -> CmdCategory {
        match self {
            Self::LeftClick
            | Self::RightClick
            | Self::MiddleClick
            | Self::Forward
            | Self::Backward
            | Self::DoubleClick
            | Self::FireButton
            | Self::EasyAim
            | Self::LedLoop
            | Self::Disabled => CmdCategory::Clicks,
            Self::DpiCycle | Self::DpiUp | Self::DpiDown => CmdCategory::Dpi,
            Self::ScrollUp | Self::ScrollDown | Self::ScrollLeft | Self::ScrollRight => {
                CmdCategory::Scroll
            }
            Self::MediaPlayer
            | Self::PlayPause
            | Self::MediaStop
            | Self::PrevTrack
            | Self::NextTrack
            | Self::VolumeUp
            | Self::VolumeDown
            | Self::Mute => CmdCategory::Media,
            Self::BrowserHome
            | Self::BrowserFavorites
            | Self::BrowserForward
            | Self::BrowserBackward
            | Self::BrowserStop
            | Self::BrowserRefresh
            | Self::BrowserSearch
            | Self::Email
            | Self::Calculator
            | Self::MyComputer => CmdCategory::Browser,
            Self::Cut
            | Self::Copy
            | Self::Paste
            | Self::Open
            | Self::Save
            | Self::Find
            | Self::Redo
            | Self::SelectAll
            | Self::Print
            | Self::CloseWindow
            | Self::SwapWindows
            | Self::ShowDesktop
            | Self::RunCommand
            | Self::LockPc
            | Self::ScreenCapture => CmdCategory::Shortcuts,
            Self::Shortcut { .. } => CmdCategory::Custom,
            Self::Macro { .. } => CmdCategory::Macro,
        }
    }

    pub fn is_macro(&self) -> bool {
        matches!(self, Self::Macro { .. })
    }

    /// The byte the original writes into the button map (`FUN_004a09f0`).
    pub fn code(&self) -> u8 {
        match self {
            Self::Disabled => 0x00,
            Self::LeftClick => 0x01,
            Self::RightClick => 0x02,
            Self::MiddleClick => 0x03,
            Self::Forward => 0x04,
            Self::Backward => 0x05,
            Self::DpiCycle => 0x06,
            Self::DpiUp => 0x07,
            Self::DpiDown => 0x08,
            Self::DoubleClick => 0x09,
            Self::FireButton => 0x0A,
            Self::ScrollUp => 0x0B,
            Self::ScrollDown => 0x0C,
            Self::ScrollLeft => 0x0D,
            Self::ScrollRight => 0x0E,
            Self::Shortcut { .. } => 0x10,
            Self::Macro { .. } => 0x11,
            Self::EasyAim => 0x16,
            Self::LedLoop => 0x17,
            Self::Cut => 0x20,
            Self::Copy => 0x21,
            Self::Paste => 0x22,
            Self::Open => 0x23,
            Self::Save => 0x24,
            Self::Find => 0x25,
            Self::Redo => 0x26,
            Self::SelectAll => 0x28,
            Self::Print => 0x29,
            Self::CloseWindow => 0x2A,
            Self::SwapWindows => 0x2B,
            Self::ShowDesktop => 0x2C,
            Self::RunCommand => 0x2D,
            Self::LockPc => 0x2E,
            Self::ScreenCapture => 0x32,
            Self::MediaPlayer => 0x40,
            Self::PlayPause => 0x41,
            Self::MediaStop => 0x42,
            Self::PrevTrack => 0x43,
            Self::NextTrack => 0x44,
            Self::VolumeUp => 0x45,
            Self::VolumeDown => 0x46,
            Self::Mute => 0x47,
            Self::BrowserHome => 0x50,
            Self::BrowserFavorites => 0x51,
            Self::BrowserForward => 0x52,
            Self::BrowserBackward => 0x53,
            Self::BrowserStop => 0x54,
            Self::BrowserRefresh => 0x55,
            Self::BrowserSearch => 0x56,
            Self::Email => 0x57,
            Self::Calculator => 0x58,
            Self::MyComputer => 0x59,
        }
    }

    pub fn all() -> Vec<Command> {
        use Command::*;
        vec![
            LeftClick,
            RightClick,
            MiddleClick,
            Forward,
            Backward,
            DoubleClick,
            FireButton,
            EasyAim,
            Disabled,
            DpiCycle,
            DpiUp,
            DpiDown,
            ScrollUp,
            ScrollDown,
            ScrollLeft,
            ScrollRight,
            MediaPlayer,
            PlayPause,
            MediaStop,
            PrevTrack,
            NextTrack,
            VolumeUp,
            VolumeDown,
            Mute,
            BrowserHome,
            BrowserFavorites,
            BrowserForward,
            BrowserBackward,
            BrowserStop,
            BrowserRefresh,
            BrowserSearch,
            Email,
            Calculator,
            MyComputer,
            Cut,
            Copy,
            Paste,
            Open,
            Save,
            Find,
            Redo,
            SelectAll,
            Print,
            CloseWindow,
            SwapWindows,
            ShowDesktop,
            RunCommand,
            LockPc,
            ScreenCapture,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmdCategory {
    Clicks,
    Dpi,
    Scroll,
    Media,
    Browser,
    Shortcuts,
    Custom,
    Macro,
}

impl CmdCategory {
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Clicks => lang.t("Clics y botones", "Clicks & buttons"),
            Self::Dpi => "DPI",
            Self::Scroll => lang.t("Desplazamiento", "Scrolling"),
            Self::Media => lang.t("Multimedia", "Media"),
            Self::Browser => lang.t("Navegador", "Browser"),
            Self::Shortcuts => lang.t("Atajos", "Shortcuts"),
            Self::Custom => lang.t("Personalizado", "Custom"),
            Self::Macro => lang.t("Macros", "Macros"),
        }
    }

    pub const ALL: [CmdCategory; 7] = [
        CmdCategory::Clicks,
        CmdCategory::Dpi,
        CmdCategory::Scroll,
        CmdCategory::Media,
        CmdCategory::Browser,
        CmdCategory::Shortcuts,
        CmdCategory::Custom,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonAssignment {
    pub command: Command,
}

impl ButtonAssignment {
    pub fn new(command: Command) -> Self {
        Self { command }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiStage {
    pub dpi: u32,
    pub color: [u8; 3],
    pub enabled: bool,
}

impl DpiStage {
    pub fn new(dpi: u32, color: [u8; 3]) -> Self {
        Self {
            dpi,
            color,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightSettings {
    /// 0..=6: the seven modes accepted by `FUN_0049e7a0`.
    pub mode: usize,
    /// 1..=8
    pub brightness: u8,
    /// 4..=8
    pub speed: u8,
    pub color: [u8; 3],
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            // FUN_004a4670 restores mode 0 (LED Off) and a green light colour.
            mode: 0,
            brightness: 8,
            speed: 6,
            color: DEFAULT_LIGHT_COLOR,
        }
    }
}

impl LightSettings {
    pub fn mode_label(&self, lang: Lang) -> &'static str {
        light_mode_label(self.mode, lang)
    }
}

pub fn light_mode_label(mode: usize, lang: Lang) -> &'static str {
    match mode {
        0 => lang.t("LED apagado", "LED off"),
        1 => lang.t("Estático", "Static"),
        2 => lang.t("Respiración", "Breathing"),
        3 => lang.t("Neón", "Neon"),
        4 => lang.t("Respiración de color", "Color breathing"),
        5 => lang.t("DPI estático", "Static DPI"),
        _ => lang.t("DPI respirando", "Breathing DPI"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// 0..=3 index into POLLING_RATES.
    pub polling: usize,
    /// 2..=25 ms
    pub debounce: u8,
    pub ripple: bool,
    pub angle_snap: bool,
    pub motion_sync: bool,
    /// 1 or 2 mm
    pub lift_mm: u8,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            polling: 3,
            debounce: 4,
            ripple: true,
            angle_snap: false,
            motion_sync: true,
            lift_mm: 1,
        }
    }
}

impl PerformanceSettings {
    pub fn polling_hz(&self) -> u32 {
        POLLING_RATES[self.polling.min(3)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerSettings {
    /// 1..=60 minutes
    pub light_off_min: u8,
    /// 1..=60 minutes
    pub sleep_min: u8,
    pub move_wake: bool,
}

impl Default for PowerSettings {
    fn default() -> Self {
        Self {
            light_off_min: 1,
            sleep_min: 10,
            // FUN_004a4670 does not set 0x93c, so a fresh profile has it off.
            move_wake: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub buttons: Vec<ButtonAssignment>,
    pub dpi: Vec<DpiStage>,
    pub light: LightSettings,
    pub performance: PerformanceSettings,
    pub power: PowerSettings,
}

impl Profile {
    pub fn principal(lang: Lang) -> Self {
        Self {
            name: lang.t("Principal", "Main").into(),
            buttons: default_buttons(),
            dpi: factory_dpi(),
            light: LightSettings::default(),
            performance: PerformanceSettings::default(),
            power: PowerSettings::default(),
        }
    }

    pub fn precision(lang: Lang) -> Self {
        let mut p = Self::principal(lang);
        p.name = lang.t("Precisión", "Precision").into();
        p.dpi = pad_factory_tail(vec![
            DpiStage::new(400, [0x38, 0xBD, 0xF8]),
            DpiStage::new(800, [0x38, 0xBD, 0xF8]),
            DpiStage::new(1_200, [0x38, 0xBD, 0xF8]),
            DpiStage::new(1_600, [0x38, 0xBD, 0xF8]),
            DpiStage::new(2_400, [0x38, 0xBD, 0xF8]),
            DpiStage::new(3_200, [0x38, 0xBD, 0xF8]),
        ]);
        p.light.mode = 1;
        p.light.color = [0x9A, 0xD8, 0xFF];
        p.performance.debounce = 2;
        p.performance.angle_snap = true;
        p.performance.ripple = true;
        p
    }

    pub fn gaming(lang: Lang) -> Self {
        let mut p = Self::principal(lang);
        p.name = lang.t("Juego", "Gaming").into();
        p.dpi = pad_factory_tail(vec![
            DpiStage::new(800, [0xFF, 0x5C, 0x4D]),
            DpiStage::new(1_600, [0xFF, 0x9F, 0x43]),
            DpiStage::new(3_200, [0xFF, 0xD1, 0x66]),
            DpiStage::new(6_400, [0x4A, 0xDE, 0x80]),
            DpiStage::new(12_000, [0x38, 0xBD, 0xF8]),
            DpiStage::new(22_000, [0xA7, 0x8B, 0xFA]),
        ]);
        p.light.mode = 2;
        p.light.brightness = 8;
        p.light.speed = 7;
        p.light.color = [0xFF, 0x3B, 0x30];
        p.performance.debounce = 3;
        p.performance.ripple = false;
        p.performance.angle_snap = false;
        p
    }

    pub fn sanitize(&mut self, lang: Lang) {
        if self.name.trim().is_empty() {
            self.name = lang.t("Perfil", "Profile").into();
        }
        while self.buttons.len() < BUTTON_COUNT {
            self.buttons.push(ButtonAssignment::new(Command::Disabled));
        }
        self.buttons.truncate(BUTTON_COUNT);
        // FUN_004a09f0 refuses a map with no Left Click; keep that rule here.
        if !self
            .buttons
            .iter()
            .any(|b| matches!(b.command, Command::LeftClick))
        {
            self.buttons[0].command = Command::LeftClick;
        }
        while self.dpi.len() < DPI_STAGE_COUNT {
            let index = self.dpi.len();
            self.dpi.push(factory_stage(index));
        }
        self.dpi.truncate(DPI_STAGE_COUNT);
        for stage in &mut self.dpi {
            stage.dpi = stage.dpi.clamp(DPI_MIN, DPI_MAX);
            stage.dpi = snap_dpi(stage.dpi);
        }
        self.light.mode = self.light.mode.min(LIGHT_MODE_COUNT - 1);
        self.light.brightness = self.light.brightness.clamp(1, 8);
        self.light.speed = self.light.speed.clamp(4, 8);
        self.performance.polling = self.performance.polling.min(3);
        self.performance.debounce = self.performance.debounce.clamp(2, 25);
        self.performance.lift_mm = self.performance.lift_mm.clamp(1, 2);
        self.power.light_off_min = self.power.light_off_min.clamp(1, 60);
        self.power.sleep_min = self.power.sleep_min.clamp(1, 60);
    }
}

/// The eight factory DPI slots from `FUN_004a4670`. Slots 7 and 8 are disabled.
pub fn factory_dpi() -> Vec<DpiStage> {
    pad_factory_tail(Vec::new())
}

/// Append the factory slots 7 and 8 (disabled) to a six-slot preset.
pub fn pad_factory_tail(mut stages: Vec<DpiStage>) -> Vec<DpiStage> {
    while stages.len() < DPI_STAGE_COUNT {
        let index = stages.len();
        stages.push(factory_stage(index));
    }
    stages
}

/// One slot of the factory table; slots 7 and 8 stay disabled.
pub fn factory_stage(index: usize) -> DpiStage {
    let position = DEFAULT_DPI_POSITIONS[index.min(DPI_STAGE_COUNT - 1)];
    let (dpi, enabled) = if position == 0 {
        // The original leaves these slots at position 0; show a usable value.
        (800 * (index as u32).saturating_sub(5).max(1), false)
    } else {
        (position_to_dpi(position), true)
    };
    let mut stage = DpiStage::new(dpi, DEFAULT_STAGE_COLORS[index.min(DPI_STAGE_COUNT - 1)]);
    stage.enabled = enabled;
    stage
}

pub fn default_buttons() -> Vec<ButtonAssignment> {
    button_hardware_ids()
        .iter()
        .map(|id| {
            // Factory byte for this hardware id, from FUN_004a4670 / DAT_004ec8f0.
            let code = DEFAULT_BUTTON_CODES[(*id as usize).saturating_sub(1)];
            ButtonAssignment::new(command_from_code(code))
        })
        .collect()
}

/// Reverse of [`Command::code`], used when reading a factory button byte.
pub fn command_from_code(code: u8) -> Command {
    match code {
        0x01 => Command::LeftClick,
        0x02 => Command::RightClick,
        0x03 => Command::MiddleClick,
        0x04 => Command::Forward,
        0x05 => Command::Backward,
        0x06 => Command::DpiCycle,
        0x07 => Command::DpiUp,
        0x08 => Command::DpiDown,
        0x09 => Command::DoubleClick,
        0x0A => Command::FireButton,
        0x0B => Command::ScrollUp,
        0x0C => Command::ScrollDown,
        0x0D => Command::ScrollLeft,
        0x0E => Command::ScrollRight,
        0x10 => Command::Shortcut {
            ctrl: false,
            shift: false,
            alt: false,
            win: false,
            key: "A".into(),
        },
        0x11 => Command::Macro {
            name: String::new(),
            mode: MacroMode::Repeat,
            loops: 1,
        },
        0x16 => Command::EasyAim,
        0x17 => Command::LedLoop,
        0x20 => Command::Cut,
        0x21 => Command::Copy,
        0x22 => Command::Paste,
        0x23 => Command::Open,
        0x24 => Command::Save,
        0x25 => Command::Find,
        0x26 => Command::Redo,
        0x28 => Command::SelectAll,
        0x29 => Command::Print,
        0x2A => Command::CloseWindow,
        0x2B => Command::SwapWindows,
        0x2C => Command::ShowDesktop,
        0x2D => Command::RunCommand,
        0x2E => Command::LockPc,
        0x32 => Command::ScreenCapture,
        0x40 => Command::MediaPlayer,
        0x41 => Command::PlayPause,
        0x42 => Command::MediaStop,
        0x43 => Command::PrevTrack,
        0x44 => Command::NextTrack,
        0x45 => Command::VolumeUp,
        0x46 => Command::VolumeDown,
        0x47 => Command::Mute,
        0x50 => Command::BrowserHome,
        0x51 => Command::BrowserFavorites,
        0x52 => Command::BrowserForward,
        0x53 => Command::BrowserBackward,
        0x54 => Command::BrowserStop,
        0x55 => Command::BrowserRefresh,
        0x56 => Command::BrowserSearch,
        0x57 => Command::Email,
        0x58 => Command::Calculator,
        0x59 => Command::MyComputer,
        _ => Command::Disabled,
    }
}

pub fn button_names(lang: Lang) -> [&'static str; BUTTON_COUNT] {
    [
        lang.t("Botón izquierdo", "Left button"),
        lang.t("Botón derecho", "Right button"),
        lang.t("Rueda / central", "Wheel / middle"),
        lang.t("Lateral delantero", "Forward side"),
        lang.t("Lateral trasero", "Back side"),
    ]
}

pub fn button_hardware_ids() -> [u8; BUTTON_COUNT] {
    // The five visible controls in ms_4_page.xml are ms_4_btn_key_1, _2, _3, _7 and _8.
    [1, 2, 3, 7, 8]
}

/// The original maps a DPI value to one of 320 slider positions:
/// below 51 -> 1, up to 10 000 -> dpi / 50, above 10 000 -> 200 + (dpi - 10 000) / 100.
/// Source: FUN_004a07d0.
pub fn dpi_to_position(dpi: u32) -> u32 {
    if dpi < 51 {
        1
    } else if dpi <= DPI_SPLIT {
        (dpi / 50).clamp(1, 200)
    } else {
        let d = dpi.min(DPI_MAX);
        200 + (d - DPI_SPLIT) / 100
    }
    .clamp(1, DPI_POSITION_MAX)
}

/// Inverse of `dpi_to_position`, used for the stage sliders.
pub fn position_to_dpi(position: u32) -> u32 {
    let p = position.clamp(1, DPI_POSITION_MAX);
    if p <= 200 {
        p * 50
    } else {
        DPI_SPLIT + (p - 200) * 100
    }
}

pub fn snap_dpi(dpi: u32) -> u32 {
    position_to_dpi(dpi_to_position(dpi))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroAction {
    KeyDown(String),
    KeyUp(String),
    MouseDown(String),
    MouseUp(String),
    Delay(u16),
    Move(i16, i16),
    Wheel(i16),
}

impl MacroAction {
    pub fn label(&self, lang: Lang) -> String {
        match self {
            Self::KeyDown(k) => format!("{} {k}", lang.t("Tecla abajo", "Key down")),
            Self::KeyUp(k) => format!("{} {k}", lang.t("Tecla arriba", "Key up")),
            Self::MouseDown(b) => format!("{} {b}", lang.t("Botón abajo", "Button down")),
            Self::MouseUp(b) => format!("{} {b}", lang.t("Botón arriba", "Button up")),
            Self::Delay(ms) => format!("{} {ms} ms", lang.t("Espera", "Delay")),
            Self::Move(x, y) => format!("{} {x}, {y}", lang.t("Mover", "Move")),
            Self::Wheel(d) => format!("{} {d}", lang.t("Rueda", "Wheel")),
        }
    }

    pub fn kind(&self, lang: Lang) -> &'static str {
        match self {
            Self::KeyDown(_) | Self::KeyUp(_) => lang.t("Tecla", "Key"),
            Self::MouseDown(_) | Self::MouseUp(_) => lang.t("Ratón", "Mouse"),
            Self::Delay(_) => lang.t("Espera", "Delay"),
            Self::Move(_, _) => lang.t("Mover", "Move"),
            Self::Wheel(_) => lang.t("Rueda", "Wheel"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<MacroAction>,
    pub mode: MacroMode,
    pub loops: u8,
}

impl Macro {
    pub fn duration_ms(&self) -> u32 {
        self.actions
            .iter()
            .map(|a| match a {
                MacroAction::Delay(ms) => *ms as u32,
                MacroAction::KeyDown(_) | MacroAction::KeyUp(_) => 30,
                MacroAction::MouseDown(_) | MacroAction::MouseUp(_) => 25,
                MacroAction::Move(_, _) => 12,
                MacroAction::Wheel(_) => 12,
            })
            .sum()
    }
}

pub fn default_macros(lang: Lang) -> Vec<Macro> {
    vec![
        Macro {
            name: lang.t("Ráfaga suave", "Soft burst").into(),
            actions: vec![
                MacroAction::MouseDown("Izq".into()),
                MacroAction::Delay(35),
                MacroAction::MouseUp("Izq".into()),
                MacroAction::Delay(90),
            ],
            mode: MacroMode::Repeat,
            loops: 3,
        },
        Macro {
            name: lang.t("Copiar y pegar", "Copy and paste").into(),
            actions: vec![
                MacroAction::KeyDown("Ctrl".into()),
                MacroAction::KeyDown("C".into()),
                MacroAction::KeyUp("C".into()),
                MacroAction::KeyUp("Ctrl".into()),
                MacroAction::Delay(140),
                MacroAction::KeyDown("Ctrl".into()),
                MacroAction::KeyDown("V".into()),
                MacroAction::KeyUp("V".into()),
                MacroAction::KeyUp("Ctrl".into()),
            ],
            mode: MacroMode::Repeat,
            loops: 1,
        },
    ]
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub lang: Lang,
    pub accent: Accent,
    pub startup: bool,
    pub animations: bool,
    pub show_tooltips: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            lang: Lang::Es,
            accent: Accent::Coral,
            startup: false,
            animations: true,
            show_tooltips: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub version: u32,
    pub profiles: Vec<Profile>,
    pub active: usize,
    pub macros: Vec<Macro>,
    pub settings: AppSettings,
}

impl Ledger {
    /// Stored-state version. 2 replaces the first preview's guessed defaults.
    pub const VERSION: u32 = 2;

    pub fn defaults(lang: Lang) -> Self {
        Self {
            version: Self::VERSION,
            profiles: vec![
                Profile::principal(lang),
                Profile::precision(lang),
                Profile::gaming(lang),
            ],
            active: 0,
            macros: default_macros(lang),
            settings: AppSettings {
                lang,
                ..Default::default()
            },
        }
    }

    /// Version 1 shipped guessed DPI and light values. Replace only profiles
    /// that still carry the untouched old defaults, so edits survive.
    fn migrate(&mut self) {
        if self.version >= Self::VERSION {
            return;
        }
        const OLD_DPI: [u32; 6] = [400, 800, 1_600, 3_200, 6_400, 10_000];
        for profile in &mut self.profiles {
            let matches_old_dpi = profile.dpi.len() >= 6
                && profile.dpi[..6]
                    .iter()
                    .zip(OLD_DPI)
                    .all(|(stage, dpi)| stage.dpi == dpi && stage.enabled)
                && profile.dpi[6..].iter().all(|stage| !stage.enabled);
            if matches_old_dpi {
                profile.dpi = factory_dpi();
            }
            if profile.light.mode == 1
                && profile.light.color == [0xFF, 0x5C, 0x4D]
                && profile.light.brightness == 8
                && profile.light.speed == 6
            {
                profile.light = LightSettings::default();
            }
        }
        self.version = Self::VERSION;
    }

    pub fn profile(&self) -> &Profile {
        &self.profiles[self.active.min(self.profiles.len() - 1)]
    }

    pub fn profile_mut(&mut self) -> &mut Profile {
        let index = self.active.min(self.profiles.len() - 1);
        &mut self.profiles[index]
    }

    pub fn sanitize(&mut self) {
        if self.profiles.is_empty() {
            self.profiles.push(Profile::principal(self.settings.lang));
        }
        if self.active >= self.profiles.len() {
            self.active = 0;
        }
        for profile in &mut self.profiles {
            profile.sanitize(self.settings.lang);
        }
        if self.macros.is_empty() {
            self.macros = default_macros(self.settings.lang);
        }
        for m in &mut self.macros {
            if m.name.trim().is_empty() {
                m.name = "Macro".into();
            }
            m.loops = m.loops.clamp(1, 99);
        }
    }

    pub fn storage_path() -> Option<PathBuf> {
        let base = if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata)
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            return None;
        };
        Some(base.join("X11Studio").join("state.json"))
    }

    pub fn load_or_default() -> Self {
        let Some(path) = Self::storage_path() else {
            return Self::defaults(Lang::Es);
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::defaults(Lang::Es);
        };
        match serde_json::from_str::<Ledger>(&text) {
            Ok(mut ledger) => {
                let migrated = ledger.version < Ledger::VERSION;
                ledger.migrate();
                ledger.sanitize();
                if migrated {
                    let _ = ledger.save();
                }
                ledger
            }
            Err(_) => Self::defaults(Lang::Es),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::storage_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)
    }
}

pub fn profile_to_json(profile: &Profile) -> String {
    serde_json::to_string_pretty(profile).unwrap_or_else(|_| "{}".into())
}

pub fn profile_from_json(text: &str) -> Option<Profile> {
    let mut profile: Profile = serde_json::from_str(text).ok()?;
    profile.sanitize(Lang::Es);
    Some(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_positions_follow_the_original_curve() {
        assert_eq!(dpi_to_position(0), 1);
        assert_eq!(dpi_to_position(50), 1);
        assert_eq!(dpi_to_position(100), 2);
        assert_eq!(dpi_to_position(400), 8);
        assert_eq!(dpi_to_position(10_000), 200);
        assert_eq!(dpi_to_position(10_100), 201);
        assert_eq!(dpi_to_position(22_000), 320);
        assert_eq!(dpi_to_position(30_000), 320);
    }

    #[test]
    fn dpi_positions_round_trip() {
        for position in [1, 2, 8, 20, 100, 200, 201, 320] {
            assert_eq!(dpi_to_position(position_to_dpi(position)), position);
        }
        assert_eq!(position_to_dpi(320), 22_000);
    }

    #[test]
    fn defaults_have_the_x11_shape() {
        let ledger = Ledger::defaults(Lang::Es);
        assert_eq!(ledger.profiles.len(), 3);
        for profile in &ledger.profiles {
            assert_eq!(profile.buttons.len(), BUTTON_COUNT);
            assert_eq!(profile.dpi.len(), DPI_STAGE_COUNT);
            assert!(profile.performance.debounce >= 2 && profile.performance.debounce <= 25);
            assert!(profile.light.brightness >= 1 && profile.light.brightness <= 8);
            assert!(
                profile
                    .buttons
                    .iter()
                    .any(|b| matches!(b.command, Command::LeftClick))
            );
        }
    }

    #[test]
    fn factory_dpi_matches_the_original_table() {
        let stages = factory_dpi();
        let values: Vec<u32> = stages.iter().map(|s| s.dpi).collect();
        assert_eq!(
            values,
            vec![800, 1_600, 2_400, 3_200, 5_000, 22_000, 800, 1_600]
        );
        assert!(stages[..6].iter().all(|s| s.enabled));
        assert!(!stages[6].enabled && !stages[7].enabled);
        assert_eq!(stages[0].color, [0x00, 0x00, 0xFF]);
        assert_eq!(stages[5].color, [0xFF, 0x00, 0xFF]);
    }

    #[test]
    fn command_bytes_match_the_original_map() {
        assert_eq!(Command::LeftClick.code(), 0x01);
        assert_eq!(Command::RightClick.code(), 0x02);
        assert_eq!(Command::MiddleClick.code(), 0x03);
        assert_eq!(Command::DpiCycle.code(), 0x06);
        assert_eq!(Command::Forward.code(), 0x04);
        assert_eq!(Command::Backward.code(), 0x05);
        assert_eq!(Command::ScrollUp.code(), 0x0B);
        assert_eq!(Command::ScrollDown.code(), 0x0C);
        assert_eq!(
            Command::Macro {
                name: "m".into(),
                mode: MacroMode::Repeat,
                loops: 1,
            }
            .code(),
            0x11
        );
        assert_eq!(
            Command::Shortcut {
                ctrl: true,
                shift: false,
                alt: false,
                win: false,
                key: "A".into(),
            }
            .code(),
            0x10
        );
        let slots = DEFAULT_BUTTON_CODES;
        assert_eq!(slots[0], 0x01);
        assert_eq!(slots[5], 0x06);
        assert_eq!(slots[16], 0x0B);
        assert_eq!(slots[17], 0x0C);
    }

    #[test]
    fn lighting_defaults_follow_fun_4a4670() {
        let light = LightSettings::default();
        assert_eq!(light.mode, 0);
        assert_eq!(light.brightness, 8);
        assert_eq!(light.speed, 6);
        assert_eq!(light.color, [0x00, 0xFF, 0x00]);
    }

    #[test]
    fn version_one_defaults_migrate_to_the_factory_values() {
        let mut ledger = Ledger::defaults(Lang::Es);
        ledger.version = 1;
        ledger.profiles[0].dpi = [400, 800, 1_600, 3_200, 6_400, 10_000]
            .iter()
            .map(|dpi| DpiStage::new(*dpi, [0xFF, 0xFF, 0xFF]))
            .collect();
        ledger.profiles[0].light = LightSettings {
            mode: 1,
            brightness: 8,
            speed: 6,
            color: [0xFF, 0x5C, 0x4D],
        };
        ledger.migrate();
        assert_eq!(ledger.version, Ledger::VERSION);
        assert_eq!(ledger.profiles[0].dpi.len(), DPI_STAGE_COUNT);
        assert_eq!(ledger.profiles[0].dpi[0].dpi, 800);
        assert_eq!(ledger.profiles[0].dpi[5].dpi, 22_000);
        assert_eq!(ledger.profiles[0].light.mode, 0);
    }

    #[test]
    fn profile_json_round_trip() {
        let profile = Profile::gaming(Lang::Es);
        let text = profile_to_json(&profile);
        let back = profile_from_json(&text).expect("profile parses");
        assert_eq!(profile, back);
    }

    #[test]
    fn macro_duration_counts_delays_and_events() {
        let mac = &default_macros(Lang::Es)[0];
        assert!(mac.duration_ms() > 0);
        assert_eq!(mac.loops, 3);
    }
}
