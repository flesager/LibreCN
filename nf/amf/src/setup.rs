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

    /// Whether NR cell `cell` (PLMN, 36-bit NR Cell Identity) is one of
    /// this gNB's: its identity starts with the gNB ID (TS 38.300 §8.2).
    pub fn has_cell(&self, cell: (Plmn, u64)) -> bool {
        match *self {
            RanNodeId::Gnb { plmn, id, bits } => {
                plmn == cell.0 && (22..=32).contains(&bits) && cell.1 >> (36 - u32::from(bits)) == u64::from(id)
            }
            RanNodeId::Other => false,
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
    fn a_cell_belongs_to_the_gnb_its_identity_starts_with() {
        let plmn = Plmn::new("999", "70").unwrap();
        // UERANSIM: nci 0x000000010 with a 32-bit gNB ID is gNB 1, cell 0.
        let gnb = RanNodeId::Gnb { plmn, id: 1, bits: 32 };
        assert!(gnb.has_cell((plmn, 0x0_0000_0010)) && gnb.has_cell((plmn, 0x0_0000_001f)));
        assert!(!gnb.has_cell((plmn, 0x0_0000_0020)), "gNB 2's");
        assert!(!gnb.has_cell((Plmn::new("001", "01").unwrap(), 0x10)), "another PLMN");
        let short = RanNodeId::Gnb { plmn, id: 5, bits: 22 };
        assert!(short.has_cell((plmn, 5 << 14 | 0x3fff)) && !RanNodeId::Other.has_cell((plmn, 0)));
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
