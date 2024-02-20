//! Serial Transport layer implementation.

use std::time::Duration;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio_serial::SerialPort;

use crate::smp_framing;

use super::{AsyncSMPTransport, Result};

pub struct AsyncSerialTransport<P: SerialPort> {
    port: P,
    buf: Vec<u8>,
}

impl<P: SerialPort + io::AsyncRead + io::AsyncWrite + Unpin> AsyncSMPTransport
    for AsyncSerialTransport<P>
{
    async fn send(&mut self, frame: &[u8]) -> Result {
        let mut encoder = smp_framing::SMPTransportEncoder::new(&frame);

        self.buf.resize(128, 0);
        while !encoder.is_complete() {
            let len = encoder.write_line(&mut self.buf)?;
            AsyncWriteExt::write_all(&mut self.port, &self.buf[..len]).await?;
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        let mut decoder = smp_framing::SMPTransportDecoder::new();
        let mut buf_reader = tokio::io::BufReader::new(&mut self.port);

        while !decoder.is_complete() {
            self.buf.clear();
            let len = buf_reader.read_until(0xa, &mut self.buf).await?;

            decoder.input_line(&self.buf[..len])?;
        }

        let resp = decoder.into_frame_payload().unwrap();
        Ok(resp)
    }

    fn set_recv_timeout(&mut self, timeout: Option<Duration>) -> Result {
        let timeout = timeout.unwrap_or(Duration::MAX);

        self.port.set_timeout(timeout)?;
        Ok(())
    }
}
