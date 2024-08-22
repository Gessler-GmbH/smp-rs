use crate::transport::error::Error;
use crate::smp::SMPFrame;

pub trait SmpTransport {
    /// send a single frame
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Error>;

    /// receive a single frame
    fn receive(&mut self) -> Result<Vec<u8>, Error>;
}

pub struct CborSmpTransport {
    pub transport: Box<dyn SmpTransport>,
}

impl CborSmpTransport {
    pub fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        self.transport.send(frame)
    }
    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        self.transport.receive()
    }

    pub fn transceive(&mut self, frame: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.transport.send(frame)?;
        self.transport.receive()
    }

    pub fn send_cbor<T: serde::Serialize>(
        &mut self,
        frame: SMPFrame<T>,
    ) -> Result<(), Error> {
        let bytes = frame.encode_with_cbor();
        self.send(bytes)
    }
    pub fn receive_cbor<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<SMPFrame<T>, Error> {
        let bytes = self.receive()?;
        let frame = SMPFrame::<T>::decode_with_cbor(&bytes)?;
        Ok(frame)
    }

    pub fn transceive_cbor<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &mut self,
        frame: SMPFrame<Req>,
    ) -> Result<SMPFrame<Resp>, Error> {
        self.send_cbor(frame)?;
        self.receive_cbor()
    }
}