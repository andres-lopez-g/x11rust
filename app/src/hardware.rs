use crate::model::{DpiStage, LightSettings, POLLING_RATES, Profile};
use hidapi::{HidApi, HidDevice};

const X11_VID: u16 = 0x1D57;
const X11_PIDS: [u16; 2] = [0xFA60, 0xFA55];
const VENDOR_USAGE_PAGE: u16 = 0x000B;

/// Small HID transport for the X11's vendor collection.
///
/// The original software talks to the mouse through HID feature reports. The
/// X11 receiver exposes a vendor collection (usage page 0x000B) that accepts
/// 8-, 9-, 15- and 56-byte reports; the app's own report shapes were captured
/// from the original driver.
pub struct X11Hardware {
    api: Option<HidApi>,
    device: Option<HidDevice>,
    pub device_name: String,
    pub last_error: Option<String>,
    pub last_action: Option<String>,
}

impl X11Hardware {
    pub fn new() -> Self {
        Self {
            api: None,
            device: None,
            device_name: String::new(),
            last_error: None,
            last_action: None,
        }
    }

    pub fn connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn connect(&mut self) -> bool {
        self.close();
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(err) => {
                self.last_error = Some(format!("No se pudo abrir HID: {err}"));
                return false;
            }
        };

        let mut chosen = None;
        for wanted_pid in X11_PIDS {
            for info in api.device_list() {
                if info.vendor_id() == X11_VID
                    && info.product_id() == wanted_pid
                    && info.usage_page() == VENDOR_USAGE_PAGE
                {
                    chosen = Some(info.path().to_owned());
                    break;
                }
            }
            if chosen.is_some() {
                break;
            }
        }

        let Some(path) = chosen else {
            self.api = Some(api);
            self.last_error = Some("No se encontró el X11 (VID 1D57, PID FA60/FA55)".into());
            return false;
        };

        let device = match api.open_path(&path) {
            Ok(device) => device,
            Err(err) => {
                self.api = Some(api);
                self.last_error = Some(format!("No se pudo abrir el X11: {err}"));
                return false;
            }
        };

        self.device_name = "X11 Wireless".into();
        self.device = Some(device);
        self.api = Some(api);
        self.last_error = None;

        // The original opens the feature channel with these two short reports.
        // If either fails the device is still usable for many reports, so keep
        // the connection and remember the warning.
        if let Err(err) = self.send_raw(&[0xA0, 0x0B, 0x08, 0, 0, 0, 0, 0]) {
            self.last_error = Some(format!("Handshake 1: {err}"));
        }
        if let Err(err) = self.send_raw(&[0xA0, 0x0A, 0x80, 0, 1, 0, 0, 0]) {
            self.last_error = Some(format!("Handshake 2: {err}"));
        }
        true
    }

    pub fn close(&mut self) {
        self.device = None;
        self.api = None;
        self.device_name.clear();
    }

    pub fn send_raw(&self, report: &[u8]) -> Result<(), String> {
        let Some(device) = self.device.as_ref() else {
            return Err("sin dispositivo".into());
        };
        device
            .send_feature_report(report)
            .map_err(|err| err.to_string())
    }

    pub fn apply_profile(&mut self, profile: &Profile) -> Result<(), String> {
        self.apply_polling(profile.performance.polling)?;
        self.apply_dpi(&profile.dpi)?;
        self.apply_lighting(&profile.light)?;
        self.last_action = Some(format!(
            "{} · {} Hz · {} etapas",
            profile.name,
            profile.performance.polling_hz(),
            profile.dpi.len()
        ));
        Ok(())
    }

    /// 06 09 01 [8/4/2/1] [complement] 00 00 00 00
    pub fn apply_polling(&self, index: usize) -> Result<(), String> {
        let value = match index.min(3) {
            0 => 0x08,
            1 => 0x04,
            2 => 0x02,
            _ => 0x01,
        };
        let report = [0x06, 0x09, 0x01, value, 0xFF - value, 0, 0, 0, 0];
        self.send_raw(&report)
    }

    /// 04 38 ... six encoded stage values ... 16-bit checksum.
    pub fn apply_dpi(&self, stages: &[DpiStage]) -> Result<(), String> {
        let mut report = [0u8; 56];
        report[0] = 0x04;
        report[1] = 0x38;
        report[2] = 0x01;
        report[3] = 0x00;
        report[4] = 0x01;
        report[5] = 0x3F;
        report[6] = 0x00;
        report[7] = 0x00;

        // Fixed tail captured from the original X11 profile report.
        let tail: [u8; 36] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xFF, 0x00, 0x00,
            0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00,
            0xFF, 0xFF, 0x40, 0x00, 0xFF, 0xFF, 0xFF, 0x02,
        ];
        report[14..50].copy_from_slice(&tail);

        for (i, stage) in stages.iter().take(6).enumerate() {
            report[8 + i] = encode_dpi(stage.dpi);
            if stage.dpi > 10_000 {
                report[6] = 0x01;
                report[7] = 0x01;
            }
        }

        let checksum = report[2..50]
            .iter()
            .fold(0u16, |acc, byte| acc.wrapping_add(*byte as u16))
            .wrapping_sub(1);
        report[50] = (checksum >> 8) as u8;
        report[51] = checksum as u8;

        self.send_raw(&report)
    }

    /// 05 0F ... lighting mode/colour report. Mode is packed in the high
    /// nibble of byte 3 (0x00 = off, 0x10 = static, 0x20 = breathing ...),
    /// matching the two reports captured from the original app.
    pub fn apply_lighting(&self, light: &LightSettings) -> Result<(), String> {
        let mut report = [0u8; 15];
        report[0] = 0x05;
        report[1] = 0x0F;
        report[2] = 0x01;
        report[3] = ((light.mode.min(7) as u8) << 4) & 0xF0;
        report[4] = 0x03;
        report[5] = 0xA8;
        report[6] = 0x00;
        report[7] = 0xFF;
        report[8] = 0x00;
        report[9] = 0x01;
        report[10] = 0x04;
        report[11] = 0x01;
        // The original keeps a small checksum in byte 12; the captured
        // static/off values differ by exactly the mode nibble.
        report[12] = 0xAFu8.wrapping_add(report[3]);
        self.send_raw(&report)
    }
}

fn encode_dpi(dpi: u32) -> u8 {
    if dpi <= 10_000 {
        (dpi as f32 * 6.0 / 256.0).round().clamp(0.0, 255.0) as u8
    } else if dpi < 22_000 {
        (dpi as f32 * 3.0 / 256.0).round().clamp(0.0, 255.0) as u8
    } else {
        0x81
    }
}

pub fn polling_label(index: usize) -> String {
    let hz = POLLING_RATES[index.min(3)];
    format!("{hz} Hz")
}
