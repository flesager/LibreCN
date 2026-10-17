//! NG Setup (TS 38.413 §8.7.1), as a decision on domain values: no ASN.1,
//! no I/O.

use crate::config::{AmfConfig, Plmn, Snssai};

/// Global identity of a RAN node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RanNodeId {
    /// gNB: PLMN, gNB ID value and its length in bits (22 to 32).
    Gnb { plmn: Plmn, id: u32, bits: u8 },
    /// ng-eNB or N3IWF: not served in this release of the AMF.
    Other,
}

impl RanNodeId {
    /// Stable 32-bit key (alarm ids, logs).
    pub fn key(&self) -> u32 {
        match self {
            RanNodeId::Gnb { id, .. } => *id,
            RanNodeId::Other => 0,
        }
    }
}

impl std::fmt::Display for RanNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RanNodeId::Gnb { plmn, id, bits } => write!(f, "gNB {plmn} {id:#x}/{bits}"),
            RanNodeId::Other => f.write_str("non-gNB RAN node"),
        }
    }
}

/// A tracking area a RAN node supports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportedTa {
    /// Tracking area code.
    pub tac: [u8; 3],
    /// PLMNs broadcast in it, with their slices.
    pub plmns: Vec<(Plmn, Vec<Snssai>)>,
}

/// The parts of an NG SETUP REQUEST the AMF decides on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupRequest {
    /// Global RAN Node ID.
    pub node: RanNodeId,
    /// RAN Node Name.
    pub name: Option<String>,
    /// Supported TA List.
    pub tas: Vec<SupportedTa>,
}

/// Why an NG Setup is refused (TS 38.413 §9.3.1.2, cause "misc").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No supported TA broadcasts a PLMN the AMF serves.
    UnknownPlmn,
}

/// Accepts a RAN node that broadcasts the AMF's PLMN in at least one TA.
pub fn decide(config: &AmfConfig, request: &SetupRequest) -> Result<(), Refusal> {
    let served = request.tas.iter().any(|ta| ta.plmns.iter().any(|(plmn, _)| *plmn == config.plmn));
    if served { Ok(()) } else { Err(Refusal::UnknownPlmn) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(plmn: Plmn) -> SetupRequest {
        SetupRequest {
            node: RanNodeId::Gnb { plmn, id: 1, bits: 32 },
            name: Some("gnb".into()),
            tas: vec![SupportedTa { tac: [0, 0, 1], plmns: vec![(plmn, vec![Snssai { sst: 1, sd: None }])] }],
        }
    }

    #[test]
    fn accepts_its_plmn_refuses_others() {
        let config = AmfConfig::from_params("amf", &serde_json::Value::Null).unwrap();
        assert_eq!(decide(&config, &request(config.plmn)), Ok(()));
        let other = Plmn::new("001", "01").unwrap();
        assert_eq!(decide(&config, &request(other)), Err(Refusal::UnknownPlmn));
        let mut both = request(other);
        both.tas[0].plmns.push((config.plmn, vec![]));
        assert_eq!(decide(&config, &both), Ok(()), "one served PLMN is enough");
    }
}
