use crate::transport::error::Error;

pub trait SmpTransport {
    /// send a single frame
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Error>;

    /// receive a single frame
    fn receive(&mut self) -> Result<Vec<u8>, Error>;
}

#[cfg(feature = "payload-cbor")]
pub mod cbor {
    use crate::smp::SmpFrame;
    use crate::transport::error::Error;
    use crate::transport::smp::SmpTransport;

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

        pub fn send_cbor<T: serde::Serialize>(&mut self, frame: &SmpFrame<T>) -> Result<(), Error> {
            let bytes = frame.encode_with_cbor();
            self.send(bytes)
        }
        pub fn receive_cbor<T: serde::de::DeserializeOwned>(
            &mut self,
            expected_sequence: Option<u8>,
        ) -> Result<SmpFrame<T>, Error> {
            let bytes = self.receive()?;
            let frame = SmpFrame::<T>::decode_with_cbor(&bytes)?;
            if let Some(expected_sequence) = expected_sequence {
                if frame.sequence != expected_sequence {
                    Err(Error::Smp(crate::SmpError::UnexpectedSeq))?;
                }
            }
            Ok(frame)
        }

        pub fn transceive_cbor<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
            &mut self,
            frame: &SmpFrame<Req>,
            check_sequence: bool,
        ) -> Result<SmpFrame<Resp>, Error> {
            self.send_cbor(frame)?;
            self.receive_cbor(check_sequence.then_some(frame.sequence))
        }
    }
}
