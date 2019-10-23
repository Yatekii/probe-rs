use super::super::{Category, Error, Request, Response, Result, Status};

#[derive(Debug)]
pub struct ResetRequest;

impl Request for ResetRequest {
    const CATEGORY: Category = Category(0x0A);

    fn to_bytes(&self, _buffer: &mut [u8], _offset: usize) -> Result<usize> {
        Ok(0)
    }
}

/// Execute: indicates whether a device specific reset sequence was executed.
#[derive(Debug)]
pub enum Execute {
    NoDeviceSpecificResetSequenceImplemented = 0,
    DeviceSpecificResetSequenceImplemented = 1,
}

impl Execute {
    pub(crate) fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0 => Ok(Execute::NoDeviceSpecificResetSequenceImplemented),
            1 => Ok(Execute::DeviceSpecificResetSequenceImplemented),
            _ => Err(Error::UnexpectedAnswer),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ResetResponse {
    pub status: Status,
    pub execute: Execute,
}

impl Response for ResetResponse {
    fn from_bytes(buffer: &[u8], offset: usize) -> Result<Self> {
        Ok(ResetResponse {
            status: Status::from_byte(buffer[offset])?,
            execute: Execute::from_byte(buffer[offset + 1])?,
        })
    }
}
