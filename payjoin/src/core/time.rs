use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bitcoin::absolute::{ConversionError, Time as BitcoinTime};
use bitcoin::consensus::encode::{Decodable, Error as EncodeError};
use bitcoin::consensus::Encodable;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Time(BitcoinTime);

impl Time {
    /// Specify a time some duration from now (e.g. an expiry time).
    pub fn from_now(duration: Duration) -> Result<Self, ConversionError> {
        SystemTime::now().checked_add(duration).unwrap_or(UNIX_EPOCH).try_into()
    }

    /// Get the current time.
    pub fn now() -> Self {
        Time::try_from(SystemTime::now()).expect("Current time should always be a valid timestamp")
    }

    fn from_unix_seconds(seconds: u32) -> Result<Self, ConversionError> {
        Ok(Time(BitcoinTime::from_consensus(seconds)?))
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, ParseTimeError> {
        use ParseTimeError::*;
        let seconds = u32::consensus_decode(&mut &bytes[..]).map_err(Decode)?; // TODO check that there are no bytes left
        Time::from_unix_seconds(seconds).map_err(Convert)
    }

    pub(crate) fn to_bytes(&self) -> [u8; 4] {
        let t = self.0.to_consensus_u32();

        let mut buf = [0u8; 4];
        t.consensus_encode(&mut &mut buf[..]).expect("encoding should never fail because all valid Time values are encodable and u32 has a known width");
        buf
    }
}

#[derive(Debug)]
pub(crate) enum ParseTimeError {
    Decode(EncodeError),
    Convert(ConversionError),
}

impl std::error::Error for ParseTimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

impl std::fmt::Display for ParseTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseTimeError::*;

        match &self {
            Decode(e) => write!(f, "invalid bytes: {e}"),
            Convert(e) => write!(f, "invalid date: {e}"),
        }
    }
}

impl TryFrom<SystemTime> for Time {
    type Error = ConversionError;
    fn try_from(val: SystemTime) -> Result<Self, ConversionError> {
        Time::from_unix_seconds(val.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as u32)
    }
}
