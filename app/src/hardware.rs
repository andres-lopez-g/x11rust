//! HID transport for the X11's vendor collection.
//!
//! The X11 exposes three collections on interface 2:
//!   * usage page `0x000B` — feature reports used to read/write settings;
//!   * usage page `0x000A` — asynchronous events (battery, DPI change, ACKs);
//!   * usage page `0x000C` — consumer-control shortcuts.
//!
//! Report layouts follow the community-reverse-engineered X11 protocol that is
//! also used by the vendor's WebHID driver.  Reports are always sent whole,
//! including the report id in byte 0.  Reads need the `0xA0` permission
//! handshake: request, poll `0xA0` until it answers `A0 01`, then read the
//! target report once.

use crate::model::{Command, Ledger, Macro, MacroAction, MacroMode, Profile};
use hidapi::{HidApi, HidDevice};
use std::thread;
use std::time::{Duration, Instant};

const X11_VID: u16 = 0x1D57;
const X11_PIDS: [u16; 2] = [0xFA60, 0xFA55];
const COMMAND_USAGE_PAGE: u16 = 0x000B;
const EVENTS_USAGE_PAGE: u16 = 0x000A;

const REPORT_DPI: u8 = 0x04;
const REPORT_LIGHT: u8 = 0x05;
const REPORT_POLLING: u8 = 0x06;
const REPORT_BUTTONS: u8 = 0x08;
const REPORT_MACRO: u8 = 0x09;
const REPORT_PROFILE: u8 = 0x0C;

const WRITE_GAP: Duration = Duration::from_millis(90);
const EVENT_POLL: Duration = Duration::from_millis(250);

/// Last battery event seen on the events collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatteryState {
    pub percent: u8,
    pub charging: bool,
    pub full: bool,
}

pub struct X11Hardware {
    api: Option<HidApi>,
    command: Option<HidDevice>,
    events: Option<HidDevice>,
    pub device_name: String,
    pub last_error: Option<String>,
    pub last_action: Option<String>,
    battery: Option<BatteryState>,
    last_event_poll: Option<Instant>,
}

impl X11Hardware {
    pub fn new() -> Self {
        Self {
            api: None,
            command: None,
            events: None,
            device_name: String::new(),
            last_error: None,
            last_action: None,
            battery: None,
            last_event_poll: None,
        }
    }

    pub fn connected(&self) -> bool {
        self.command.is_some()
    }

    pub fn battery(&self) -> Option<BatteryState> {
        self.battery
    }

    /// Open the command and event collections.  Both must be present on the
    /// receiver; if the event collection cannot be opened the settings still
    /// work, only battery updates are lost.
    pub fn connect(&mut self) -> bool {
        self.close();
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(err) => {
                self.last_error = Some(format!("No se pudo abrir HID: {err}"));
                return false;
            }
        };

        let mut command_path = None;
        let mut events_path = None;
        let mut product = String::new();
        for wanted_pid in X11_PIDS {
            for info in api.device_list() {
                if info.vendor_id() != X11_VID || info.product_id() != wanted_pid {
                    continue;
                }
                match info.usage_page() {
                    COMMAND_USAGE_PAGE if command_path.is_none() => {
                        command_path = Some(info.path().to_owned());
                    }
                    EVENTS_USAGE_PAGE if events_path.is_none() => {
                        events_path = Some(info.path().to_owned());
                    }
                    _ => {}
                }
            }
            if command_path.is_some() {
                break;
            }
        }

        let Some(path) = command_path else {
            self.api = Some(api);
            self.last_error = Some("No se encontró el X11 (VID 1D57, PID FA60/FA55)".into());
            return false;
        };

        let command = match api.open_path(&path) {
            Ok(device) => device,
            Err(err) => {
                self.api = Some(api);
                self.last_error = Some(format!("No se pudo abrir el X11: {err}"));
                return false;
            }
        };

        if let Ok(Some(name)) = command.get_product_string() {
            product = name;
        }

        if let Some(path) = events_path {
            if let Ok(device) = api.open_path(&path) {
                let _ = device.set_blocking_mode(false);
                self.events = Some(device);
            }
        }

        self.device_name = if product.trim().is_empty() {
            "X11 Wireless".into()
        } else {
            product.trim().to_string()
        };
        self.command = Some(command);
        self.api = Some(api);
        self.last_error = None;
        self.last_action = None;
        self.battery = None;
        self.last_event_poll = None;
        true
    }

    pub fn close(&mut self) {
        self.command = None;
        self.events = None;
        self.api = None;
        self.device_name.clear();
    }

    pub fn send_raw(&self, report: &[u8]) -> Result<(), String> {
        let Some(device) = self.command.as_ref() else {
            return Err("sin dispositivo".into());
        };
        match device.send_feature_report(report) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Drain the events collection and keep the newest battery event.
    /// Reports look like `03 <device> 40|41 <state> <percent>`.
    pub fn poll_events(&mut self) {
        if let Some(last) = self.last_event_poll {
            if last.elapsed() < EVENT_POLL {
                return;
            }
        }
        self.last_event_poll = Some(Instant::now());
        let Some(device) = self.events.as_ref() else {
            return;
        };
        let mut buffer = [0u8; 32];
        for _ in 0..16 {
            match device.read_timeout(&mut buffer, 0) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if let Some(state) = parse_battery(&buffer[..n]) {
                        self.battery = Some(state);
                    }
                }
            }
        }
    }

    /// Request one feature report using the `0xA0` read-permission handshake.
    pub fn read_report(&self, id: u8, len: u8) -> Result<Vec<u8>, String> {
        let Some(device) = self.command.as_ref() else {
            return Err("sin dispositivo".into());
        };
        for _ in 0..5 {
            device
                .send_feature_report(&[0xA0, id, len, 0, 1, 0, 0, 0])
                .map_err(|err| err.to_string())?;
            for _ in 0..8 {
                thread::sleep(Duration::from_millis(90));
                let mut ready = [0u8; 65];
                ready[0] = 0xA0;
                match device.get_feature_report(&mut ready) {
                    Ok(n) if n > 1 && ready[0] == 0xA0 && ready[1] == 1 => {
                        thread::sleep(Duration::from_millis(60));
                        let mut data = [0u8; 65];
                        data[0] = id;
                        match device.get_feature_report(&mut data) {
                            Ok(n) if n >= len as usize => {
                                let report = data[..len as usize].to_vec();
                                if report[0] != id {
                                    return Err(format!(
                                        "respuesta inesperada: id 0x{:02X}",
                                        report[0]
                                    ));
                                }
                                return Ok(report);
                            }
                            Ok(n) => return Err(format!("respuesta corta: {n} bytes")),
                            Err(err) => return Err(err.to_string()),
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(format!("sin respuesta al informe 0x{id:02X}"))
    }

    /// Read the main settings blocks and return them as readable hex, for the
    /// `hid_probe` tool and for diagnosing a device.
    pub fn read_snapshot(&self) -> Result<String, String> {
        let mut out = String::new();
        for (name, id, len) in [
            ("DPI", REPORT_DPI, 0x38u8),
            ("Polling", REPORT_POLLING, 0x09),
            ("Lighting", REPORT_LIGHT, 0x0F),
            ("Buttons", REPORT_BUTTONS, 0x3B),
            ("Profile", REPORT_PROFILE, 0x0A),
        ] {
            thread::sleep(Duration::from_millis(320));
            match self.read_report(id, len) {
                Ok(report) => {
                    out.push_str(&format!("{name} (0x{id:02X}): {report:02X?}\n"));
                }
                Err(err) => out.push_str(&format!("{name} (0x{id:02X}): error {err}\n")),
            }
        }
        Ok(out)
    }

    /// Apply a profile to the mouse: sensor/DPI, polling, lighting and power,
    /// button map, onboard macros and then the active profile slot.
    pub fn apply_ledger(&mut self, ledger: &Ledger) -> Result<(), String> {
        if !self.connected() {
            return Err("sin dispositivo".into());
        }
        let profile = ledger.profile();
        let profile_id = hardware_profile_id(ledger);
        let mut errors: Vec<String> = Vec::new();

        let reports: [(u8, Vec<u8>, &str); 3] = [
            (REPORT_DPI, dpi_report(profile, profile_id), "DPI"),
            (REPORT_POLLING, polling_report(profile), "sondeo"),
            (REPORT_LIGHT, lighting_report(profile), "iluminación"),
        ];
        for (id, report, label) in reports {
            if let Err(err) = self.send_raw(&report) {
                errors.push(format!("{label}: {err}"));
            } else {
                let _ = id;
                thread::sleep(WRITE_GAP);
            }
        }

        for (index, mac) in ledger.macros.iter().enumerate().take(255) {
            match macro_packets(mac, (index + 1) as u8) {
                Ok(packets) => {
                    for packet in packets {
                        if let Err(err) = self.send_raw(&packet) {
                            errors.push(format!("macro {}: {err}", mac.name));
                            break;
                        }
                        thread::sleep(WRITE_GAP);
                    }
                }
                Err(err) => errors.push(format!("macro {}: {err}", mac.name)),
            }
        }

        match button_report(profile, ledger, profile_id) {
            Ok(report) => {
                if let Err(err) = self.send_raw(&report) {
                    errors.push(format!("botones: {err}"));
                } else {
                    thread::sleep(WRITE_GAP);
                }
            }
            Err(err) => errors.push(format!("botones: {err}")),
        }

        if let Err(err) = self.send_raw(&profile_report(ledger)) {
            errors.push(format!("perfil: {err}"));
        }

        self.last_action = Some(format!(
            "{} · {} Hz · {} etapas · perfil {}",
            profile.name,
            profile.performance.polling_hz(),
            profile.dpi.iter().filter(|stage| stage.enabled).count(),
            profile_id
        ));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Compatibility helper used by small tools/tests.
    pub fn apply_profile(&mut self, profile: &Profile) -> Result<(), String> {
        let ledger = Ledger {
            version: Ledger::VERSION,
            profiles: vec![profile.clone()],
            active: 0,
            macros: Vec::new(),
            settings: crate::model::AppSettings::default(),
        };
        self.apply_ledger(&ledger)
    }
}

fn hardware_profile_id(ledger: &Ledger) -> u8 {
    (ledger.active.min(4) + 1) as u8
}

fn parse_battery(report: &[u8]) -> Option<BatteryState> {
    if report.len() < 5 || report[0] != 0x03 {
        return None;
    }
    if report[2] != 0x40 && report[2] != 0x41 {
        return None;
    }
    let state = report[3];
    let percent = report[4].min(100);
    Some(BatteryState {
        percent,
        charging: state == 0x03,
        full: state == 0x02 || state == 0x01,
    })
}

// ---------------------------------------------------------------------------
// Pure report builders (unit tested without a device)
// ---------------------------------------------------------------------------

/// Report 0x04: DPI stages, sensor options and stage colours.
pub fn dpi_report(profile: &Profile, profile_id: u8) -> Vec<u8> {
    let mut report = [0u8; 56];
    report[0] = REPORT_DPI;
    report[1] = 0x38;
    report[2] = profile_id.max(1);

    let lod = match profile.performance.lift_mm {
        2 => 1u8,
        _ => 0u8,
    };
    report[3] = (lod << 4) | u8::from(profile.performance.angle_snap);
    report[4] =
        (u8::from(profile.performance.motion_sync) << 4) | u8::from(profile.performance.ripple);

    let mut mask = 0u8;
    for (index, stage) in profile.dpi.iter().take(8).enumerate() {
        if stage.enabled {
            mask |= 1 << index;
        }
    }
    // At least one stage must stay enabled for the mouse to be usable.
    report[5] = if mask == 0 { 0x01 } else { mask };

    let mut double_flags = 0u8;
    let mut triple_flags = 0u8;
    for (index, stage) in profile.dpi.iter().take(8).enumerate() {
        let (low, high, double, triple) = dpi_wire(stage.dpi);
        report[8 + index] = low;
        report[16 + index] = high;
        if double {
            double_flags |= 1 << index;
        }
        if triple {
            triple_flags |= 1 << index;
        }
        if index < 8 {
            report[25 + index * 3] = stage.color[0];
            report[26 + index * 3] = stage.color[1];
            report[27 + index * 3] = stage.color[2];
        }
    }
    report[6] = double_flags;
    report[7] = triple_flags;

    let active = profile.active_stage.min(7) + 1;
    report[24] = active as u8;
    report[49] = 0x02; // static DPI-change indicator

    let checksum: u16 = report[3..=49].iter().map(|byte| *byte as u16).sum();
    report[50] = (checksum >> 8) as u8;
    report[51] = checksum as u8;
    report.to_vec()
}

/// Report 0x06: polling rate period byte and its complement.
pub fn polling_report(profile: &Profile) -> Vec<u8> {
    let code = match profile.performance.polling.min(3) {
        0 => 0x08u8,
        1 => 0x04,
        2 => 0x02,
        _ => 0x01,
    };
    vec![REPORT_POLLING, 0x09, 0x01, code, 0xFF - code, 0, 0, 0, 0]
}

/// Report 0x05: lighting, deep sleep, regular sleep, key response and colour.
pub fn lighting_report(profile: &Profile) -> Vec<u8> {
    let mut report = [0u8; 15];
    report[0] = REPORT_LIGHT;
    report[1] = 0x0F;
    report[2] = 0x01;
    report[3] = ((profile.light.mode.min(6) as u8) & 0x0F) << 4;

    let deep_sleep = profile.power.sleep_min.clamp(1, 60);
    report[4] = (deep_sleep & 0xF0) | led_speed_code(profile.light.speed);
    report[5] = ((deep_sleep & 0x0F) << 4) | profile.light.brightness.clamp(1, 8);

    report[6] = profile.light.color[0];
    report[7] = profile.light.color[1];
    report[8] = profile.light.color[2];

    // The light-off timer is the shorter sleep stage and is stored directly.
    report[9] = profile.power.light_off_min.clamp(1, 60);
    // Key response is stored in the same millisecond value the profile uses.
    report[10] = profile.performance.debounce.clamp(2, 25);

    let checksum: u16 = report[3..=10].iter().map(|byte| *byte as u16).sum();
    report[11] = (checksum >> 8) as u8;
    report[12] = checksum as u8;
    report.to_vec()
}

/// Report 0x0C: switch to the active profile and announce the profile count.
pub fn profile_report(ledger: &Ledger) -> Vec<u8> {
    let max = ledger.profiles.len().clamp(1, 5) as u8;
    let current = hardware_profile_id(ledger).min(max);
    vec![
        REPORT_PROFILE,
        0x0A,
        current,
        !current,
        max,
        !max,
        0,
        0,
        0,
        0,
    ]
}

/// Report 0x08: all 18 button slots, 3 bytes each.
pub fn button_report(profile: &Profile, ledger: &Ledger, profile_id: u8) -> Result<Vec<u8>, String> {
    let mut report = [0u8; 59];
    report[0] = REPORT_BUTTONS;
    report[1] = 0x3B;
    report[2] = profile_id.max(1);

    // Factory map, in wire (firmware-action) values.  The device itself
    // returns 0x00 for an unused slot, so that is what we write.
    let mut slots: [[u8; 3]; 18] = [[0x00, 0, 0]; 18];
    slots[0] = [0x02, 0, 0]; // left
    slots[1] = [0x03, 0, 0]; // right
    slots[2] = [0x04, 0, 0]; // middle
    slots[5] = [0x0D, 0, 0]; // DPI cycle
    slots[6] = [0x06, 0, 0]; // forward
    slots[7] = [0x05, 0, 0]; // backward
    slots[16] = [0x09, 0, 0]; // scroll up
    slots[17] = [0x0A, 0, 0]; // scroll down

    for (index, assignment) in profile.buttons.iter().take(5).enumerate() {
        let target = [0usize, 1, 2, 6, 7][index];
        slots[target] = command_slot(&assignment.command, ledger)?;
    }

    for (index, slot) in slots.iter().enumerate() {
        report[3 + index * 3] = slot[0];
        report[4 + index * 3] = slot[1];
        report[5 + index * 3] = slot[2];
    }

    let checksum: u16 = report[3..=56].iter().map(|byte| *byte as u16).sum();
    report[57] = (checksum >> 8) as u8;
    report[58] = checksum as u8;
    Ok(report.to_vec())
}

fn command_slot(command: &Command, ledger: &Ledger) -> Result<[u8; 3], String> {
    use Command::*;
    let simple = match command {
        Disabled => Some(0x00),
        LeftClick => Some(0x02),
        RightClick => Some(0x03),
        MiddleClick => Some(0x04),
        Backward => Some(0x05),
        Forward => Some(0x06),
        DoubleClick => Some(0x07),
        FireButton => Some(0x08),
        ScrollUp => Some(0x09),
        ScrollDown => Some(0x0A),
        DpiCycle => Some(0x0D),
        DpiUp => Some(0x0E),
        DpiDown => Some(0x0F),
        EasyAim => Some(0x10),
        LedLoop => Some(0x29),
        MediaPlayer => Some(0x15),
        PrevTrack => Some(0x16),
        NextTrack => Some(0x17),
        PlayPause => Some(0x18),
        MediaStop => Some(0x19),
        Mute => Some(0x1A),
        VolumeUp => Some(0x1B),
        VolumeDown => Some(0x1C),
        Email => Some(0x1E),
        Calculator => Some(0x1D),
        MyComputer => Some(0x23),
        BrowserFavorites => Some(0x1F),
        BrowserForward => Some(0x20),
        BrowserBackward => Some(0x21),
        BrowserStop => Some(0x22),
        BrowserRefresh => Some(0x24),
        BrowserHome => Some(0x25),
        BrowserSearch => Some(0x26),
        _ => None,
    };
    if let Some(action) = simple {
        return Ok([action, 0, 0]);
    }

    let (ctrl, shift, alt, win, key) = match command {
        Shortcut {
            ctrl,
            shift,
            alt,
            win,
            key,
        } => (*ctrl, *shift, *alt, *win, key.as_str()),
        Cut => (true, false, false, false, "X"),
        Copy => (true, false, false, false, "C"),
        Paste => (true, false, false, false, "V"),
        Open => (true, false, false, false, "O"),
        Save => (true, false, false, false, "S"),
        Find => (true, false, false, false, "F"),
        Redo => (true, false, false, false, "Y"),
        SelectAll => (true, false, false, false, "A"),
        Print => (true, false, false, false, "P"),
        CloseWindow => (false, false, true, false, "F4"),
        SwapWindows => (false, false, true, false, "Tab"),
        ShowDesktop => (false, false, false, true, "D"),
        RunCommand => (false, false, false, true, "R"),
        LockPc => (false, false, false, true, "L"),
        ScreenCapture => (false, false, false, false, "PrintScreen"),
        Macro { name, .. } => {
            let id = ledger
                .macros
                .iter()
                .position(|mac| mac.name == *name)
                .ok_or_else(|| format!("la macro «{name}» no existe"))?;
            return Ok([0x12, 0, (id + 1).min(255) as u8]);
        }
        ScrollLeft | ScrollRight => {
            return Err("el X11 no admite desplazamiento lateral".into());
        }
        _ => return Err(format!("acción sin traducción HID: {}", command.label(crate::model::Lang::Es))),
    };

    let usage = key_usage(key)
        .ok_or_else(|| format!("tecla no compatible con el ratón: «{key}»"))?;
    let modifiers = u8::from(ctrl)
        | (u8::from(shift) << 1)
        | (u8::from(alt) << 2)
        | (u8::from(win) << 3);
    Ok([0x11, modifiers, usage])
}

fn key_usage(key: &str) -> Option<u8> {
    let key = key.trim();
    let upper = key.to_ascii_uppercase();
    if upper.len() == 1 {
        let ch = upper.as_bytes()[0];
        if ch.is_ascii_uppercase() {
            return Some(0x04 + ch - b'A');
        }
        if ch.is_ascii_digit() {
            return Some(if ch == b'0' { 0x27 } else { 0x1E + ch - b'1' });
        }
    }
    let usage = match upper.as_str() {
        "SPACE" | "SPACEBAR" => 0x2C,
        "ENTER" | "RETURN" => 0x28,
        "ESCAPE" | "ESC" => 0x29,
        "TAB" => 0x2B,
        "BACKSPACE" => 0x2A,
        "DELETE" | "DEL" => 0x4C,
        "INSERT" => 0x49,
        "HOME" => 0x4A,
        "END" => 0x4D,
        "PAGEUP" => 0x4B,
        "PAGEDOWN" => 0x4E,
        "ARROWLEFT" | "LEFT" => 0x50,
        "ARROWRIGHT" | "RIGHT" => 0x4F,
        "ARROWUP" | "UP" => 0x52,
        "ARROWDOWN" | "DOWN" => 0x51,
        "PRINTSCREEN" | "PRTSC" => 0x46,
        "CAPSLOCK" => 0x39,
        "-" => 0x2D,
        "=" => 0x2E,
        "[" => 0x2F,
        "]" => 0x30,
        "\\" => 0x31,
        ";" => 0x33,
        "'" => 0x34,
        "`" => 0x35,
        "," => 0x36,
        "." => 0x37,
        "/" => 0x38,
        _ => {
            if let Some(rest) = upper.strip_prefix('F') {
                if let Ok(number) = rest.parse::<u8>() {
                    if (1..=12).contains(&number) {
                        return Some(0x3A + number - 1);
                    }
                }
            }
            return None;
        }
    };
    Some(usage)
}

/// Report 0x09 (three pages): onboard macro definition.
pub fn macro_packets(mac: &Macro, id: u8) -> Result<[Vec<u8>; 3], String> {
    let mut actions: Vec<u8> = Vec::new();
    let mut count: u16 = 0;
    let mut pending_delay: u16 = 0;

    for action in &mac.actions {
        match action {
            MacroAction::Delay(ms) => {
                pending_delay = pending_delay.saturating_add(*ms);
                continue;
            }
            MacroAction::Move(_, _) | MacroAction::Wheel(_) => continue,
            _ => {}
        }
        let (pressed, code) = match action {
            MacroAction::KeyDown(key) => (true, key_usage(key)),
            MacroAction::KeyUp(key) => (false, key_usage(key)),
            MacroAction::MouseDown(button) => (true, mouse_code(button)),
            MacroAction::MouseUp(button) => (false, mouse_code(button)),
            _ => continue,
        };
        let code = code.ok_or_else(|| format!("paso no compatible en «{}»", mac.name))?;
        let before = actions.len();
        let units = pending_delay / 10;
        if units <= 127 {
            actions.push(u8::from(!pressed) << 7 | units as u8);
            actions.push(code);
            count += 1;
        } else {
            let blocks = (pending_delay / 200).min(250) as u8;
            let remainder = ((pending_delay % 200) / 10) as u8;
            actions.push(u8::from(!pressed) << 7 | remainder);
            actions.push(code);
            actions.push(blocks);
            actions.push(0x03);
            count += 2;
        }
        if actions.len() > 100 {
            actions.truncate(before);
            count = count.saturating_sub(if units <= 127 { 1 } else { 2 });
            break;
        }
        pending_delay = 0;
    }

    let mut first = vec![0u8; 64];
    first[0] = REPORT_MACRO;
    first[1] = 0x40;
    first[2] = id;
    first[3] = 0x00;
    first[4] = match mac.mode {
        MacroMode::Repeat => 0x00,
        MacroMode::UntilKey => 0x01,
        MacroMode::HoldRelease => 0x02,
    };
    first[8] = mac.loops.max(1);
    let name = mac.name.as_bytes();
    let name_len = name.len().min(20);
    first[9..9 + name_len].copy_from_slice(&name[..name_len]);
    first[29] = count.min(255) as u8;
    let page0 = actions.len().min(34);
    first[30..30 + page0].copy_from_slice(&actions[..page0]);

    let mut second = vec![0u8; 64];
    second[0] = REPORT_MACRO;
    second[1] = 0x40;
    second[2] = id;
    second[3] = 0x01;
    let page1 = actions.len().saturating_sub(34).min(60);
    if page1 > 0 {
        second[4..4 + page1].copy_from_slice(&actions[34..34 + page1]);
    }

    let mut third = vec![0u8; 64];
    third[0] = REPORT_MACRO;
    third[1] = 0x40;
    third[2] = id;
    third[3] = 0x02;
    let page2 = actions.len().saturating_sub(94).min(6);
    if page2 > 0 {
        third[4..4 + page2].copy_from_slice(&actions[94..94 + page2]);
    }

    let mut checksum: u16 = 0;
    for byte in &first[4..] {
        checksum = checksum.wrapping_add(*byte as u16);
    }
    for byte in &second[4..] {
        checksum = checksum.wrapping_add(*byte as u16);
    }
    for byte in &third[4..9] {
        checksum = checksum.wrapping_add(*byte as u16);
    }
    third[10] = (checksum >> 8) as u8;
    third[11] = checksum as u8;

    Ok([first, second, third])
}

fn mouse_code(button: &str) -> Option<u8> {
    let button = button.to_ascii_lowercase();
    match button.as_str() {
        "izq" | "izquierdo" | "left" | "l" => Some(0xF1),
        "der" | "derecho" | "right" | "r" => Some(0xF2),
        "med" | "medio" | "middle" | "m" => Some(0xF3),
        "atras" | "atrás" | "back" | "backward" => Some(0xF4),
        "adelante" | "forward" => Some(0xF5),
        _ => None,
    }
}

/// Lighting speed is shown as 4..8 in the original profile, while the report
/// field uses the documented 1..5 scale.
fn led_speed_code(speed: u8) -> u8 {
    speed.clamp(4, 8) - 3
}

/// DPI 3311 transfer table (50..12000 DPI in the sensor's 50/100 steps).
const DPI_3311: [u8; 220] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x10, 0x11, 0x12,
    0x13, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x22, 0x23, 0x24, 0x25,
    0x26, 0x27, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x37, 0x38,
    0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x4a, 0x4b,
    0x4c, 0x4d, 0x4e, 0x4f, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5e,
    0x5f, 0x60, 0x61, 0x62, 0x63, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70,
    0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7f, 0x80, 0x81, 0x82, 0x83,
    0x84, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x93, 0x94, 0x95, 0x96,
    0x97, 0x98, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa7, 0xa8, 0xa9,
    0xaa, 0xab, 0xac, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xbb, 0xbc,
    0xbd, 0xbe, 0xbf, 0xc0, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdd, 0xde, 0xdf, 0xe0, 0xe1,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xea, 0xeb, 0x76, 0x77, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d,
];

/// Encode a DPI value as the sensor's (low, high) pair and range flags.
fn dpi_wire(dpi: u32) -> (u8, u8, bool, bool) {
    let value = dpi.clamp(50, 22_000);
    let mut effective = value;
    let mut double = false;
    let mut triple = false;
    if (24_100..=25_000).contains(&value) {
        effective = ((value as f32 / 3.0).round() as u32).clamp(8_000, 8_500);
        triple = true;
    } else if value > 24_000 {
        effective = 8_500;
    } else if value > 12_000 {
        effective = (value as f32 / 2.0).round() as u32;
        double = true;
    }
    let rounded = if effective <= 10_000 {
        ((effective as f32 / 50.0).round() * 50.0) as u32
    } else {
        ((effective as f32 / 100.0).round() * 100.0) as u32
    };
    let high = u8::from(rounded > 10_000);
    let index = if rounded <= 10_000 {
        (rounded.saturating_sub(50) / 50) as usize
    } else {
        (200 + (rounded.saturating_sub(10_100) / 100)) as usize
    };
    let low = DPI_3311.get(index).copied().unwrap_or(0x8D);
    (low, high, double, triple)
}

pub fn polling_label(index: usize) -> String {
    let hz = crate::model::POLLING_RATES[index.min(3)];
    format!("{hz} Hz")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Lang, Ledger};

    fn profile() -> Profile {
        Profile::principal(Lang::Es)
    }

    #[test]
    fn dpi_table_has_every_supported_step() {
        assert_eq!(DPI_3311.len(), 220);
        assert_eq!(dpi_wire(800), (0x12, 0, false, false));
        assert_eq!(dpi_wire(1600), (0x25, 0, false, false));
        assert_eq!(dpi_wire(2400), (0x38, 0, false, false));
        assert_eq!(dpi_wire(3200), (0x4B, 0, false, false));
        assert_eq!(dpi_wire(5000), (0x75, 0, false, false));
        assert_eq!(dpi_wire(22_000), (0x81, 1, true, false));
    }

    #[test]
    fn dpi_report_matches_the_wire_layout() {
        let report = dpi_report(&profile(), 1);
        assert_eq!(report.len(), 56);
        assert_eq!(&report[..3], &[0x04, 0x38, 0x01]);
        assert_eq!(report[3], 0x00);
        assert_eq!(report[4], 0x11); // motion sync + ripple both on
        assert_eq!(report[5], 0x3F);
        assert_eq!(report[24], 2); // factory active stage is the second
        assert_eq!(&report[25..28], &[0xFF, 0x00, 0x00]);
        assert_eq!(&report[28..31], &[0x00, 0xFF, 0x00]);
        let expected: u16 = report[3..=49].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((report[50] as u16) << 8) | report[51] as u16, expected);
    }

    #[test]
    fn polling_values_match_the_period_bytes() {
        let mut p = profile();
        for (index, code) in [(0, 0x08), (1, 0x04), (2, 0x02), (3, 0x01)] {
            p.performance.polling = index;
            assert_eq!(polling_report(&p), vec![0x06, 0x09, 0x01, code, 0xFF - code, 0, 0, 0, 0]);
        }
    }

    #[test]
    fn lighting_report_keeps_every_user_value() {
        let mut p = profile();
        p.light = crate::model::LightSettings {
            mode: 2,
            brightness: 7,
            speed: 8,
            color: [0x12, 0x34, 0x56],
        };
        p.power.light_off_min = 1;
        p.power.sleep_min = 10;
        p.performance.debounce = 4;
        let report = lighting_report(&p);
        assert_eq!(report.len(), 15);
        assert_eq!(report[3], 0x20);
        assert_eq!(report[4] & 0x0F, 5); // speed 8 -> wire 5
        assert_eq!(report[4] & 0xF0, 0x00);
        assert_eq!(report[5] & 0x0F, 7);
        assert_eq!(report[5] & 0xF0, 0xA0); // 10 min deep sleep
        assert_eq!(&report[6..9], &[0x12, 0x34, 0x56]);
        assert_eq!(report[9], 0x01);
        assert_eq!(report[10], 0x04);
        let expected: u16 = report[3..=10].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((report[11] as u16) << 8) | report[12] as u16, expected);
    }

    #[test]
    fn button_report_preserves_the_factory_map() {
        let ledger = Ledger::defaults(Lang::Es);
        let report = button_report(ledger.profile(), &ledger, 1).expect("buttons encode");
        assert_eq!(report.len(), 59);
        assert_eq!(&report[3..6], &[0x02, 0, 0]);
        assert_eq!(&report[6..9], &[0x03, 0, 0]);
        assert_eq!(&report[9..12], &[0x04, 0, 0]);
        assert_eq!(&report[18..21], &[0x0D, 0, 0]);
        assert_eq!(&report[21..24], &[0x06, 0, 0]);
        assert_eq!(&report[24..27], &[0x05, 0, 0]);
        assert_eq!(&report[51..54], &[0x09, 0, 0]);
        assert_eq!(&report[54..57], &[0x0A, 0, 0]);
        let expected: u16 = report[3..=56].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((report[57] as u16) << 8) | report[58] as u16, expected);
    }

    #[test]
    fn shortcuts_use_keyboard_firmware_action() {
        let ledger = Ledger::defaults(Lang::Es);
        let slot = command_slot(
            &Command::Shortcut {
                ctrl: true,
                shift: false,
                alt: false,
                win: false,
                key: "c".into(),
            },
            &ledger,
        )
        .expect("shortcut encodes");
        assert_eq!(slot, [0x11, 0x01, 0x06]);
        let slot = command_slot(
            &Command::Shortcut {
                ctrl: false,
                shift: false,
                alt: false,
                win: false,
                key: "F5".into(),
            },
            &ledger,
        )
        .expect("function key encodes");
        assert_eq!(slot, [0x11, 0x00, 0x3E]);
    }

    #[test]
    fn macro_packets_have_the_documented_checksum() {
        let mac = &crate::model::default_macros(Lang::Es)[0];
        let packets = macro_packets(mac, 1).expect("macro encodes");
        assert_eq!(packets[0].len(), 64);
        assert_eq!(packets[0][0], 0x09);
        assert_eq!(packets[0][3], 0x00);
        assert_eq!(packets[1][3], 0x01);
        assert_eq!(packets[2][3], 0x02);
        assert_eq!(packets[0][29], 2); // one press and one release
        let mut sum: u16 = 0;
        for byte in &packets[0][4..] {
            sum = sum.wrapping_add(*byte as u16);
        }
        for byte in &packets[1][4..] {
            sum = sum.wrapping_add(*byte as u16);
        }
        for byte in &packets[2][4..9] {
            sum = sum.wrapping_add(*byte as u16);
        }
        assert_eq!(((packets[2][10] as u16) << 8) | packets[2][11] as u16, sum);
    }

    #[test]
    fn captured_device_reports_validate_against_our_checksums() {
        // Read from the connected X11 (1D57:FA60) with `hid_probe`.
        let dpi: [u8; 56] = [
            0x04, 0x38, 0x01, 0x00, 0x00, 0x1F, 0x20, 0x20, 0x09, 0x12, 0x25, 0x4B, 0x75, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x03, 0xFF, 0x00, 0x00,
            0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00,
            0xFF, 0xFF, 0x80, 0x00, 0xFF, 0xFF, 0xFF, 0x02, 0x0F, 0x59, 0x00, 0x00, 0x00, 0x00,
        ];
        let sum: u16 = dpi[3..=49].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((dpi[50] as u16) << 8) | dpi[51] as u16, sum);

        let light: [u8; 15] = [
            0x05, 0x0F, 0x01, 0x00, 0x03, 0xA8, 0x00, 0xFF, 0x00, 0x01, 0x04, 0x01, 0xAF, 0x00,
            0x00,
        ];
        let sum: u16 = light[3..=10].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((light[11] as u16) << 8) | light[12] as u16, sum);

        let buttons: [u8; 59] = [
            0x08, 0x3B, 0x01, 0x02, 0x00, 0x00, 0x03, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x06, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x0A,
            0x00, 0x00, 0x00, 0x34,
        ];
        let sum: u16 = buttons[3..=56].iter().map(|byte| *byte as u16).sum();
        assert_eq!(((buttons[57] as u16) << 8) | buttons[58] as u16, sum);
    }

    #[test]
    fn battery_events_are_parsed() {
        assert_eq!(parse_battery(&[0x03, 0x55, 0x40, 0x01, 0x45]), Some(BatteryState {
            percent: 69,
            charging: false,
            full: true,
        }));
        assert_eq!(parse_battery(&[0x03, 0x55, 0x41, 0x03, 0x64]), Some(BatteryState {
            percent: 100,
            charging: true,
            full: false,
        }));
        assert_eq!(parse_battery(&[0x03, 0x55, 0x40, 0x01]), None);
        assert_eq!(parse_battery(&[0x02, 0x55, 0x40, 0x01, 0x45]), None);
    }
}
