use hidapi::HidApi;

fn main() {
    let api = HidApi::new().expect("hidapi");
    for dev_info in api
        .device_list()
        .filter(|d| d.vendor_id() == 0x1D57 && d.product_id() == 0xFA60)
    {
        let label = format!(
            "iface={} usage_page=0x{:04X} usage=0x{:04X}",
            dev_info.interface_number(),
            dev_info.usage_page(),
            dev_info.usage()
        );
        let Ok(dev) = dev_info.open_device(&api) else {
            continue;
        };
        let mut desc = [0u8; hidapi::MAX_REPORT_DESCRIPTOR_SIZE];
        if let Ok(n) = dev.get_report_descriptor(&mut desc) {
            println!("{label}: descriptor {n} bytes: {:02X?}", &desc[..n]);
        }
        for report_id in 0u8..=20 {
            let mut buf = [0u8; 65];
            buf[0] = report_id;
            match dev.get_feature_report(&mut buf) {
                Ok(n) if n > 1 => {
                    let nonzero = buf[..n].iter().filter(|b| **b != 0).count();
                    println!(
                        "{label}: id={report_id} n={n} nonzero={nonzero} data={:02X?}",
                        &buf[..n.min(20)]
                    );
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
