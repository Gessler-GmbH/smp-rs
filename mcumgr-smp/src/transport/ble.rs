use nom::HexDisplay;
use std::{future::Future, pin::Pin, task::Poll};

use btleplug::api::{Peripheral, ValueNotification, WriteType};
use tracing::*;

use super::{AsyncSMPTransport, Result};

pub struct BLETransport {
    pub peripheral: btleplug::platform::Peripheral,
    pub characteristic: btleplug::api::Characteristic,
    pub notifications: Pin<Box<dyn futures::Stream<Item = ValueNotification>>>,
    pub mtu: usize,
}

impl BLETransport {}

impl AsyncSMPTransport for BLETransport {
    fn poll_send(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        frame: &[u8],
    ) -> core::task::Poll<Result> {
        let me = self.get_mut();

        for chunk in frame.chunks(me.mtu) {
            debug!("Writing chunk via ble");
            trace!("\n{}", chunk.to_hex(16));
            core::task::ready!(me
                .peripheral
                .write(&me.characteristic, chunk, WriteType::WithoutResponse)
                .as_mut()
                .poll(cx))
            .map_err(error_to_io)?;
        }

        std::task::Poll::Ready(Ok(()))
    }

    fn poll_receive(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Vec<u8>>> {
        let me = self.get_mut();
        let mut ret = Vec::new();

        let notifications = core::task::ready!(me.notifications.as_mut().poll_next(cx)).unwrap();
        debug!("received {} bytes", notifications.value.len());

        ret.extend(notifications.value);

        let data_len = u16::from_be_bytes([ret[2], ret[3]]);
        if me.mtu < ret.len() {
            me.mtu = ret.len();
            debug!("updating mtu to {}", me.mtu);
        }
        let data_len = data_len + 8;

        debug!("Expecting {} bytes", data_len);

        while ret.len() < data_len as usize {
            trace!("Waiting on next chunk");
            let notifications =
                core::task::ready!(me.notifications.as_mut().poll_next(cx)).unwrap();
            ret.extend(notifications.value);
            debug!("Got {}/{} bytes", ret.len(), data_len);
        }

        trace!("\n{}", ret.to_hex(16));
        Poll::Ready(Ok(ret))
    }
}

fn error_to_io(err: btleplug::Error) -> std::io::Error {
    todo!()
}
