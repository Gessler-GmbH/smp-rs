use crate::smp::SMPFrame;
use crate::transport::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait SmpTransportAsync {
    /// send a single frame
    async fn send(&mut self, frame: Vec<u8>) -> Result<(), Error>;

    /// receive a single frame
    async fn receive(&mut self) -> Result<Vec<u8>, Error>;
}

pub struct CborSmpTransportAsync {
    pub transport: Box<dyn SmpTransportAsync>,
}

impl CborSmpTransportAsync {
    pub async fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        self.transport.send(frame).await
    }
    pub async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        self.transport.receive().await
    }

    pub async fn transceive(&mut self, frame: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.transport.send(frame).await?;
        self.transport.receive().await
    }

    pub async fn send_cbor<T: serde::Serialize>(
        &mut self,
        frame: SMPFrame<T>,
    ) -> Result<(), Error> {
        let bytes = frame.encode_with_cbor();
        self.send(bytes).await
    }
    pub async fn receive_cbor<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<SMPFrame<T>, Error> {
        let bytes = self.receive().await?;
        let frame = SMPFrame::<T>::decode_with_cbor(&bytes)?;
        Ok(frame)
    }

    pub async fn transceive_cbor<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &mut self,
        frame: SMPFrame<Req>,
    ) -> Result<SMPFrame<Resp>, Error> {
        self.send_cbor(frame).await?;
        self.receive_cbor().await
    }
}
