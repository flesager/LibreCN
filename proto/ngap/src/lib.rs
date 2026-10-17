//! NGAP, the NG Application Protocol of the N2 interface (3GPP TS 38.413),
//! ASN.1 Aligned PER.
//!
//! [`generated`] is produced by `codegen/` from `asn1/<version>/*.asn` (see
//! `asn1/fetch.py`); do not edit it. Open types stay undecoded (`Any`) until
//! asked for: a message is decoded in steps, and only the IEs actually used.
//!
//! ```ignore
//! let pdu = librecn_ngap::decode(bytes)?;                     // NGAP-PDU
//! if let NGAPPDU::initiatingMessage(m) = &pdu {
//!     if m.procedure_code == ID_NG_SETUP {
//!         let req: NGSetupRequest = librecn_ngap::open(&m.value)?; // the procedure
//!         for ie in &req.protocol_ies.0 {
//!             if ie.id == ID_GLOBAL_RAN_NODE_ID.0 {
//!                 let node: GlobalRANNodeID = librecn_ngap::open(&ie.value)?; // one IE
//!             }
//!         }
//!     }
//! }
//! ```

/// Version of TS 38.413 the bindings were generated from.
pub const VERSION: &str = "15.13.0";

#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod generated;
#[allow(clippy::all)]
mod procedures;

pub use procedures::{check, procedure_name};

pub use generated::ngap_common_data_types::*;
pub use generated::ngap_constants::*;
pub use generated::ngap_ies::*;
pub use generated::ngap_pdu_contents::*;
pub use generated::ngap_pdu_descriptions::*;
pub use rasn::types::Any;

/// A decoding failure (never a panic: the input comes from outside).
pub type DecodeError = rasn::error::DecodeError;
/// An encoding failure.
pub type EncodeError = rasn::error::EncodeError;

/// Decodes an NGAP PDU (the payload of an SCTP message with PPID 60).
pub fn decode(bytes: &[u8]) -> Result<NGAPPDU, DecodeError> {
    rasn::aper::decode(bytes)
}

/// Encodes an NGAP PDU.
pub fn encode(pdu: &NGAPPDU) -> Result<Vec<u8>, EncodeError> {
    rasn::aper::encode(pdu)
}

/// Procedure code of a PDU; `None` for a PDU kind added by a later release.
pub fn procedure_code(pdu: &NGAPPDU) -> Option<u8> {
    match pdu {
        NGAPPDU::initiatingMessage(m) => Some(m.procedure_code.0),
        NGAPPDU::successfulOutcome(m) => Some(m.procedure_code.0),
        NGAPPDU::unsuccessfulOutcome(m) => Some(m.procedure_code.0),
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

/// Decodes an open type (a procedure's message, an IE value) into `T`.
pub fn open<T: rasn::Decode>(value: &Any) -> Result<T, DecodeError> {
    rasn::aper::decode(value.as_bytes())
}

/// Encodes `value` as an open type.
pub fn wrap<T: rasn::Encode>(value: &T) -> Result<Any, EncodeError> {
    rasn::aper::encode(value).map(Any::new)
}

/// Where [`check`] found a value that does not survive decoding and re-encoding.
#[derive(Debug)]
pub enum CheckError {
    /// The value does not decode as its type.
    Decode(&'static str, DecodeError),
    /// The decoded value does not encode.
    Encode(&'static str, EncodeError),
    /// Re-encoding gives other bytes.
    Differs(&'static str),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::Decode(what, e) => write!(f, "{what}: decode: {e}"),
            CheckError::Encode(what, e) => write!(f, "{what}: encode: {e}"),
            CheckError::Differs(what) => write!(f, "{what}: re-encoding differs"),
        }
    }
}

/// IE ids are `u16` in some generated containers, `ProtocolIEID` in others.
trait IeId {
    fn raw(&self) -> u16;
}

impl IeId for u16 {
    fn raw(&self) -> u16 {
        *self
    }
}

impl IeId for ProtocolIEID {
    fn raw(&self) -> u16 {
        self.0
    }
}

/// Decodes `value` as a `T`, re-encodes it and compares.
fn round_trip<T: rasn::Decode + rasn::Encode>(what: &'static str, value: &Any) -> Result<T, CheckError> {
    let decoded: T = open(value).map_err(|e| CheckError::Decode(what, e))?;
    let encoded = rasn::aper::encode(&decoded).map_err(|e| CheckError::Encode(what, e))?;
    if encoded != value.as_bytes() {
        return Err(CheckError::Differs(what));
    }
    Ok(decoded)
}
