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
    pub async fn adapters() -> Result<Vec<Adapter>, Error> {
        let manager = Manager::new().await?;
        manager.adapters().await.map_err(Error::BLE)
    }

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
            if let Some(res) = self
                .notifications
                .next()
                .await
                .filter(|notif| notif.uuid == SMP_CHAR)
            {
                return Ok(res.value);
            }
        }
    }
}
