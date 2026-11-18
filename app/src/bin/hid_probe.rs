#[path = "../hardware.rs"]
mod hardware;
#[path = "../model.rs"]
mod model;

use std::thread;
use std::time::Duration;

fn main() {
    let mut hw = hardware::X11Hardware::new();
    println!("connect: {}", hw.connect());
    println!("device: {}", hw.device_name);
    println!("error: {:?}", hw.last_error);
    if !hw.connected() {
        std::process::exit(1);
    }

    // Give the receiver a moment to push its first event, then show what the
    // app can read without touching the mouse configuration.
    for _ in 0..6 {
        hw.poll_events();
        thread::sleep(Duration::from_millis(250));
    }
    match hw.battery() {
        Some(battery) => println!(
            "battery: {}% charging={} full={}",
            battery.percent, battery.charging, battery.full
        ),
        None => println!("battery: no event yet"),
    }

    match hw.read_snapshot() {
        Ok(snapshot) => println!("{snapshot}"),
        Err(err) => println!("read snapshot error: {err}"),
    }

    if std::env::args().any(|arg| arg == "--write-test") {
        write_roundtrip(&hw);
    }
}

/// Read every settings report, write the exact same bytes back and read it
/// again.  This validates the write path without changing any setting.
fn write_roundtrip(hw: &hardware::X11Hardware) {
    println!("--- write round-trip (no value changes) ---");
    let mut reports: Vec<(&str, u8, u8, Vec<u8>)> = Vec::new();
    for (name, id, len) in [
        ("dpi", 0x04u8, 0x38u8),
        ("polling", 0x06, 0x09),
        ("lighting", 0x05, 0x0F),
        ("buttons", 0x08, 0x3B),
    ] {
        match hw.read_report(id, len) {
            Ok(report) => reports.push((name, id, len, report)),
            Err(err) => println!("{name}: read failed: {err}"),
        }
        thread::sleep(Duration::from_millis(320));
    }

    let profile = hw.read_report(0x0C, 0x0A).ok();
    if let Some(read) = &profile {
        let current = read.get(3).copied().unwrap_or(1);
        let max = read.get(5).copied().unwrap_or(1);
        let write = vec![0x0C, 0x0A, current, !current, max, !max, 0, 0, 0, 0];
        reports.push(("profile", 0x0C, 0x0A, write));
    }

    for (name, _id, _len, report) in &reports {
        match hw.send_raw(report) {
            Ok(()) => println!("{name}: wrote {} bytes", report.len()),
            Err(err) => println!("{name}: write failed: {err}"),
        }
        thread::sleep(Duration::from_millis(250));
    }

    thread::sleep(Duration::from_millis(500));
    for (name, id, len, expected) in &reports {
        match hw.read_report(*id, *len) {
            Ok(actual) => {
                if *name == "profile" {
                    let same = actual.get(3) == expected.get(2)
                        && actual.get(5) == expected.get(4);
                    println!("{name}: {}", if same { "OK" } else { "DIFF" });
                    if !same {
                        println!("  expected current/max {:?}/{:?}",
                            expected.get(2), expected.get(4));
                        println!("  actual   current/max {:?}/{:?}",
                            actual.get(3), actual.get(5));
                    }
                } else if actual == *expected {
                    println!("{name}: OK");
                } else {
                    println!("{name}: DIFF");
                    println!("  expected {expected:02X?}");
                    println!("  actual   {actual:02X?}");
                }
            }
            Err(err) => println!("{name}: read-back failed: {err}"),
        }
        thread::sleep(Duration::from_millis(320));
    }
}
