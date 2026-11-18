#[path = "../hardware.rs"]
mod hardware;
#[path = "../model.rs"]
mod model;

use model::{Lang, Ledger};

fn main() {
    let mut hw = hardware::X11Hardware::new();
    println!("connect: {}", hw.connect());
    println!("name: {} error: {:?}", hw.device_name, hw.last_error);
    let ledger = Ledger::defaults(Lang::Es);
    let profile = ledger.profile().clone();
    println!(
        "profile: {} dpi={:?} polling={} light={}",
        profile.name,
        profile.dpi.iter().map(|s| s.dpi).collect::<Vec<_>>(),
        profile.performance.polling_hz(),
        profile.light.mode
    );
    match hw.apply_profile(&profile) {
        Ok(()) => println!("apply: OK"),
        Err(err) => println!("apply: ERR {err}"),
    }
    println!("last_action: {:?}", hw.last_action);
}
