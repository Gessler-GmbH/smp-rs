// Author: Egor Markov <mark_ee@live.com>

use super::{error::Error, smp::SmpTransportAsync};
use async_trait::async_trait;
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use futures::{Stream, StreamExt};
use std::{pin::Pin, thread, time::Duration};
use uuid::{uuid, Uuid};

pub const SMP_CHAR: Uuid = uuid!("DA2E7828-FBCE-4E01-AE9E-261174997C48");

pub struct BleTransport {
    peripheral_device: Peripheral,
    smp_char: Characteristic,
    notifications: Pin<Box<dyn Stream<Item = btleplug::api::ValueNotification> + Send>>,
}

impl BleTransport {
    pub async fn new(name: String, scan_timeout: Duration) -> Result<Self, Error> {
        Self::connect(name, scan_timeout).await
    }

    async fn connect(name: String, scan_timeout: Duration) -> Result<Self, Error> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters.into_iter().nth(0).unwrap();

        adapter.start_scan(ScanFilter::default()).await?;
        thread::sleep(scan_timeout);
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

        let peripheral_device = peripheral_device.unwrap();

        peripheral_device.connect().await?;
        peripheral_device.discover_services().await?;
        let smp_char = peripheral_device
            .characteristics()
            .into_iter()
            .find(|attr| attr.uuid == SMP_CHAR)
            .unwrap();

        peripheral_device.subscribe(&smp_char).await?;

        let notifications = peripheral_device.notifications().await?;

        Ok(Self {
            peripheral_device,
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
        let res = self.notifications.next().await.unwrap();
        Ok(res.value)
    }
}
