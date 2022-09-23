use std::{sync::Arc, thread::sleep, time::Duration};

use anyhow::{Context, Result};
use embedded_svc::{
    event_bus::EventBus,
    httpd::{registry::Registry, Body, Handler, Method, Response},
    wifi::{AccessPointConfiguration, AuthMethod, Configuration, Wifi},
};
use esp_idf_svc::{
    httpd::ServerRegistry,
    netif::{EspNetifStack, IpEvent},
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
    wifi::{EspWifi, WifiEvent},
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

fn main() -> Result<()> {
    let wifi_config = Configuration::AccessPoint(AccessPointConfiguration {
        ssid: "System.ESP32-C3".into(),
        ssid_hidden: false,
        auth_method: AuthMethod::WPA2Personal,
        password: "red1337!".into(),
        ..Default::default()
    });

    let sys_loop = Arc::new(EspSysLoopStack::new().context("Failed to get event loop")?);

    let mut wifi = EspWifi::new(
        Arc::new(EspNetifStack::new()?),
        sys_loop.clone(),
        Arc::new(EspDefaultNvs::new()?),
    )
    .unwrap();
    println!("Capabilities: {:?}", wifi.get_capabilities()?);
    wifi.set_configuration(&wifi_config)?;

    println!(
        "Current directory metadata: {:?}",
        std::env::current_dir().unwrap().metadata()
    );

    println!("Getting partitions");

    // Oh boy
    unsafe {
        use esp_idf_sys as idf;
        use esp_idf_sys::*;

        let mut it = esp_partition_find(
            esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
            esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
            std::ptr::null(),
        );

        let mut i = 0;
        while it != std::ptr::null_mut() {
            let data = *esp_partition_get(it);
            println!("Partition {}", i);

            println!(
                "\ttype: {} {}",
                data.type_,
                match data.type_ as c_types::c_uint {
                    idf::esp_partition_type_t_ESP_PARTITION_TYPE_DATA => "DATA",
                    idf::esp_partition_type_t_ESP_PARTITION_TYPE_ANY => "ANY",
                    idf::esp_partition_type_t_ESP_PARTITION_TYPE_APP => "APP",
                    _ => "ERR_UNKNOWN",
                }
            );

            println!("\tsize: {}", human_readable_byte_size(data.size));
            println!(
                "\tsubtype: {}",
                match data.subtype {
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_FACTORY => "FACTORY",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_MIN
                        ..=idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_MAX => "OTA",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_COREDUMP =>
                        "DATA COREDUMP",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_FAT => "DATA FAT",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_EFUSE_EM =>
                        "DATA EFUSE EM",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_ESPHTTPD =>
                        "DATA ESPHTTPD",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_NVS => "DATA NVS",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_NVS_KEYS =>
                        "DATA NVS KEYS",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_PHY => "DATA PHY",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_SPIFFS => "DATA SPIFFS",
                    idf::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_UNDEFINED =>
                        "DATA UNDEFINED",
                    _ => "UNKNOWN",
                }
            );
            println!("\taddress: {}", data.address);

            it = esp_partition_next(it);
            i += 1;
        }
    }

    let _s = sys_loop
        .get_loop()
        .clone()
        .subscribe(move |event: &WifiEvent| println!("Event received: {:?}", event))
        .context("Failed to subscribe to wifi")?;

    let _i = sys_loop
        .get_loop()
        .clone()
        .subscribe(move |event: &IpEvent| println!("Event received: {:?}", event))
        .context("Failed to subscribe to ip")?;

    let _server = ServerRegistry::new()
        .handler(Handler::new("/test", Method::Get, |r| {
            Response::new(200)
                .body(Body::Bytes(
                    format!("Lmao nice: {}", r.query_string().unwrap_or("".to_owned()))
                        .to_owned()
                        .into_bytes(),
                ))
                .into()
        }))?
        .start(&esp_idf_svc::httpd::Configuration {
            http_port: 80,
            ..Default::default()
        });

    loop {
        sleep(Duration::from_millis(1000));
    }
}

fn human_readable_byte_size(num: u32) -> String {
    let mut n = num;
    let mut level = 0;

    while n > 1023 {
        n = n / 1024;
        level += 1;
    }

    format!(
        "{}{}",
        n,
        match level {
            0 => "B",
            1 => "KB",
            2 => "MB",
            3 => "GB",
            _ => "TOO BIG TO HANDLE BABY",
        }
    )
}
