// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use base64::engine::general_purpose;
use base64::Engine;
use crc::Crc;
use std::cmp::min;
use thiserror::Error;

/// there are multiple possible CRC implementations. This matches the results from mcumgr
const CALC_CRC: Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

#[derive(Error, Debug)]
pub enum SMPTransportError {
    #[error("unexpected frame")]
    UnexpectedFrame,
    #[error("unknown frame start: {0:?}")]
    UnknownFrameStart([u8; 2]),
    #[error("packet length invalid")]
    PacketLength(u16, usize),
    #[error("wrong crc")]
    CRCError,
    #[error("base64 decoding error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("base64 encoding error: {0}")]
    Base64EncodeError(#[from] base64::EncodeSliceError),
}

pub struct SMPTransportDecoder {
    /// length + 2 bytes CRC
    content_length: u16,
    buf: Vec<u8>,
}

impl Default for SMPTransportDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SMPTransportDecoder {
    pub fn new() -> Self {
        Self {
            content_length: 0,
            buf: Vec::with_capacity(127),
        }
    }

    /// attempt to parse a packet from the input buffer and return whether the frame is complete
    pub fn input_line(&mut self, input: &[u8]) -> Result<bool, SMPTransportError> {
        let start = (input[0], input[1]);
        let base64_packet = general_purpose::STANDARD.decode(&input[2..input.len() - 1])?;

        let packet_body = match start {
            (0x06, 0x09) => {
                if self.content_length > 0 {
                    return Err(SMPTransportError::UnexpectedFrame);
                }

                self.content_length = u16::from_be_bytes([base64_packet[0], base64_packet[1]]);

                &base64_packet[2..]
            }
            (0x04, 0x14) => {
                if self.content_length == 0 {
                    return Err(SMPTransportError::UnexpectedFrame);
                }
                &base64_packet
            }
            (a, b) => return Err(SMPTransportError::UnknownFrameStart([a, b])),
        };

        let total_len = self.buf.len() + packet_body.len();
        if total_len > self.content_length as usize {
            return Err(SMPTransportError::PacketLength(
                self.content_length,
                total_len,
            ));
        }

        self.buf.extend_from_slice(packet_body);

        Ok(self.is_complete())
    }

    pub fn is_complete(&self) -> bool {
        self.content_length != 0 && self.content_length as usize >= self.buf.len()
    }

    pub fn into_frame_payload(self) -> Result<Vec<u8>, SMPTransportError> {
        if self.buf.len() < 2 || self.buf.len() != self.content_length as usize {
            return Err(SMPTransportError::PacketLength(
                self.content_length,
                self.buf.len(),
            ));
        }

        let mut body = self.buf;
        let crc = u16::from_be_bytes([body[body.len() - 2], body[body.len() - 1]]);
        body.truncate(body.len() - 2);

        let mut digest = CALC_CRC.digest();
        digest.update(&body);
        let crc_result = digest.finalize();

        if crc != crc_result {
            return Err(SMPTransportError::CRCError);
        }

        Ok(body)
    }
}

pub struct SMPTransportEncoder<'a> {
    written_len: usize,
    payload: &'a [u8],
}

impl<'a> SMPTransportEncoder<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        Self {
            written_len: 0,
            payload,
        }
    }

    /// Write the next line for the given payload to the supplied buffer.   
    /// returns an error if out_buf is smaller than 127 bytes
    pub fn write_line(&mut self, out_buf: &mut [u8]) -> Result<usize, SMPTransportError> {
        // max 127 with header and newline in base64 encoding
        const MAX_RAW_BODY_LEN: usize = 93; // 124.0 / 4.0 * 3.0 as usize;
        let mut base64_payload = Vec::with_capacity(MAX_RAW_BODY_LEN);

        // println!(
        //     "write_line: Written: {}; Total: {}",
        //     self.written_len,
        //     self.payload.len()
        // );

        if self.written_len == 0 {
            out_buf[0] = 0x06;
            out_buf[1] = 0x09;
            base64_payload.extend_from_slice(&(self.payload.len() as u16 + 2).to_be_bytes());
        } else {
            out_buf[0] = 0x04;
            out_buf[1] = 0x14;
        }

        let remaining_len = self.payload.len() - self.written_len;
        let last_frame = remaining_len <= MAX_RAW_BODY_LEN - base64_payload.len() - 2;

        let payload_len = if last_frame {
            remaining_len
        } else {
            min(MAX_RAW_BODY_LEN - base64_payload.len(), remaining_len)
        };

        base64_payload
            .extend_from_slice(&self.payload[self.written_len..self.written_len + payload_len]);
        self.written_len += payload_len;

        if last_frame {
            let mut digest = CALC_CRC.digest();
            digest.update(self.payload);
            let crc_result = digest.finalize();
            // println!(
            //     "appending crc16 {:x?} to last frame",
            //     &crc_result.to_be_bytes()
            // );
            base64_payload.extend_from_slice(&crc_result.to_be_bytes());
        }

        // println!("base64 payload: {:x?}", base64_payload);

        let base64_len =
            general_purpose::STANDARD.encode_slice(base64_payload, &mut out_buf[2..])?;

        // println!(
        //     "resulting base64 string: {}",
        //     String::from_utf8(Vec::from(&out_buf[2..2 + base64_len])).unwrap()
        // );

        assert!(base64_len <= 124);

        out_buf[2 + base64_len] = 0x0a; // newline

        Ok(base64_len + 3)
    }

    pub fn is_complete(&self) -> bool {
        self.written_len >= self.payload.len()
    }
}
