use std::{sync::Arc, thread::sleep, time::Duration};

use anyhow::{Context, Result};
use embedded_svc::{
    event_bus::EventBus,
    httpd::{registry::Registry, Body, Handler, Method, Response},
    wifi::{AccessPointConfiguration, AuthMethod, Configuration, Wifi},
};
use esp_idf_svc::{
    httpd::ServerRegistry,
    netif::EspNetifStack,
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

    let _s = sys_loop
        .get_loop()
        .clone()
        .subscribe(move |event: &WifiEvent| println!("Event received: {:?}", event))
        .context("Failed to subscribe to wifi")?;

    let _server = ServerRegistry::new()
        .handler(Handler::new("test", Method::Get, |r| {
            Response::new(200)
                .body(Body::Bytes("Lmao nice".to_owned().into_bytes()))
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
