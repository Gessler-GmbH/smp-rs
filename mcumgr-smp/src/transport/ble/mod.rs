// Author: Egor Markov <mark_ee@live.com>

use super::{error::Error, smp::SmpTransportAsync};
use async_trait::async_trait;
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use futures::{Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio::time::sleep;
use uuid::{uuid, Uuid};

pub const SMP_CHAR: Uuid = uuid!("DA2E7828-FBCE-4E01-AE9E-261174997C48");

pub struct BleTransport {
    peripheral_device: Peripheral,
    smp_char: Characteristic,
    notifications: Pin<Box<dyn Stream<Item = btleplug::api::ValueNotification> + Send>>,
}

impl BleTransport {
    /// Scan host system for BLE adapters. User should decide which one should be used.
    /// Usually there will be the only one, so you can take the first.
    /// Will return empty vec if there is no BLE.
    /// Will return multiple instances in case when device have internal adapter and
    /// additional one, connected by USB or UART.
    pub async fn adapters() -> Result<Vec<Adapter>, Error> {
        let manager = Manager::new().await?;
        manager.adapters().await.map_err(Error::BLE)
    }

    /// Starts listening advertizing packets for selected duration.
    /// After that allows to find peripheral device by advertized name.
    /// Unfortunatelly, MacOS and iOS doesn't allow access to BD-addresses
    /// of peripheral devices, so name filtering is the only way.
    pub async fn new(
        name: String,
        adapter: &Adapter,
        scan_timeout: Duration,
    ) -> Result<Self, Error> {
        adapter.start_scan(ScanFilter::default()).await?;
        sleep(scan_timeout).await;
        adapter.stop_scan().await?;

        let mut peripheral_device = None;
        for pd in adapter.peripherals().await? {
            if let Some(props) = pd.properties().await? {
                if props.local_name == Some(name.to_owned()) {
                    peripheral_device = Some(pd);
                    break;
                }
            }
        }

        let peripheral_device =
            peripheral_device.ok_or(Error::BLE(btleplug::Error::DeviceNotFound))?;

        peripheral_device.connect().await?;
        peripheral_device.discover_services().await?;
        let smp_char = peripheral_device
            .characteristics()
            .into_iter()
            .find(|attr| attr.uuid == SMP_CHAR)
            .ok_or(Error::BLE(btleplug::Error::NoSuchCharacteristic))?;

        peripheral_device.subscribe(&smp_char).await?;

        let notifications = peripheral_device.notifications().await?;

        Ok(Self {
            peripheral_device,
            notifications,
            smp_char,
        })
    }

    /// A bit more flexible than new()
    /// Allows user to perform scan with additional parameters,
    /// implemented by himself. For example - Scan filtering by the list of
    /// advertized services.
    pub async fn from_peripheral(device: Peripheral) -> Result<Self, Error> {
        device.connect().await?;
        device.discover_services().await?;
        let smp_char = device
            .characteristics()
            .into_iter()
            .find(|attr| attr.uuid == SMP_CHAR)
            .ok_or(Error::BLE(btleplug::Error::NoSuchCharacteristic))?;

        device.subscribe(&smp_char).await?;

        let notifications = device.notifications().await?;

        Ok(Self {
            peripheral_device: device,
            notifications,
            smp_char,
        })
    }
}

#[async_trait]
impl SmpTransportAsync for BleTransport {
    async fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        self.peripheral_device
            .write(
                &self.smp_char,
                &frame,
                btleplug::api::WriteType::WithoutResponse,
            )
            .await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        loop {
            match self.notifications.next().await {
                Some(res) if res.uuid == SMP_CHAR => return Ok(res.value),
                Some(_) => continue,
                None => {
                    return Err(Error::BLE(btleplug::Error::RuntimeError(String::from(
                        "Notification stream error",
                    ))));
                }
            }
        }
    }
}
