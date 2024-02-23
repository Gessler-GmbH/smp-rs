//! Serial Transport layer implementation.

use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use std::time::Duration;

use tokio::io::{self, AsyncBufReadExt};
use tokio_serial::{SerialPort, SerialPortBuilderExt};

use crate::smp_framing;

use super::{AsyncSMPTransport, Result};

pub struct AsyncSerialTransport<P> {
    port: P,
    buf: Vec<u8>,
}

impl AsyncSerialTransport<tokio_serial::SerialStream> {
    pub fn open<'a>(path: impl Into<std::borrow::Cow<'a, str>>, baud_rate: u32) -> Result<Self> {
        let port = tokio_serial::new(path, baud_rate).open_native_async()?;

        Ok(Self {
            port,
            buf: Vec::with_capacity(128),
        })
    }
}

impl<P: io::AsyncRead + io::AsyncWrite + Unpin> AsyncSMPTransport for AsyncSerialTransport<P> {
    fn poll_send(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        frame: &[u8],
    ) -> Poll<Result> {
        let mut encoder = smp_framing::SMPTransportEncoder::new(&frame);

        let me = self.get_mut();

        me.buf.resize(128, 0);
        while !encoder.is_complete() {
            let len = encoder.write_line(&mut me.buf)?;
            core::task::ready!(Pin::new(&mut me.port).poll_write(cx, &me.buf[..len]))?;
        }
        Poll::Ready(Ok(()))
    }

    fn poll_receive(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Result<Vec<u8>>> {
        let mut decoder = smp_framing::SMPTransportDecoder::new();

        let me = self.get_mut();

        let mut buf_reader = tokio::io::BufReader::new(&mut me.port);

        while !decoder.is_complete() {
            me.buf.clear();
            let mut task = buf_reader.read_until(0xa, &mut me.buf);
            let task = core::pin::pin!(task);
            let len = core::task::ready!(task.poll(cx))?;

            decoder.input_line(&me.buf[..len])?;
        }

        let resp = decoder.into_frame_payload().unwrap();
        Poll::Ready(Ok(resp))
    }
}
