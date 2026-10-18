//! N2 SM information: the NGAP transfers the SMF writes for the gNB and
//! reads from it (TS 38.413 §9.3.4). The only module seeing ASN.1.

use std::net::Ipv4Addr;

use librecn_ngap::*;
use rasn::types::{BitString, FixedOctetString, Integer};

/// What the gNB needs to set up a session: the UPF's tunnel endpoint and
/// the default flow.
pub struct Setup {
    pub upf: Ipv4Addr,
    pub teid: u32,
    pub qfi: u8,
    pub five_qi: u8,
    /// ARP priority level (1 to 15).
    pub arp: u8,
    /// Session AMBR in bit/s: uplink, downlink.
    pub ambr_bps: (u64, u64),
}

/// PDU Session Resource Setup Request Transfer.
pub fn setup_request_transfer(s: &Setup) -> Result<Vec<u8>, String> {
    use AnonymousPDUSessionResourceSetupRequestTransferProtocolIEsCriticality::reject;
    let ambr = PDUSessionAggregateMaximumBitRate::new(
        BitRate(Integer::from(s.ambr_bps.1)),
        BitRate(Integer::from(s.ambr_bps.0)),
        None,
    );
    let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
        TransportLayerAddress(BitString::from_slice(&s.upf.octets())),
        GTPTEID(FixedOctetString::new(s.teid.to_be_bytes())),
        None,
    ));
    let qos = QosFlowLevelQosParameters::new(
        QosCharacteristics::nonDynamic5QI(NonDynamic5QIDescriptor::new(
            FiveQI(Integer::from(s.five_qi)),
            None,
            None,
            None,
            None,
        )),
        AllocationAndRetentionPriority::new(
            PriorityLevelARP(s.arp),
            PreEmptionCapability::shall_not_trigger_pre_emption,
            PreEmptionVulnerability::not_pre_emptable,
            None,
        ),
        None,
        None,
        None,
        None,
    );
    let flows = QosFlowSetupRequestList(vec![QosFlowSetupRequestItem::new(
        QosFlowIdentifier(Integer::from(s.qfi)),
        qos,
        None,
        None,
    )]);
    let ie = |id: ProtocolIEID, value: Result<Any, EncodeError>| {
        value
            .map(|v| AnonymousPDUSessionResourceSetupRequestTransferProtocolIEs::new(id.0, reject, v))
            .map_err(|e| format!("encoding: {e}"))
    };
    let ies = vec![
        ie(ID_PDUSESSION_AGGREGATE_MAXIMUM_BIT_RATE, wrap(&ambr))?,
        ie(ID_UL_NGU_UP_TNLINFORMATION, wrap(&tunnel))?,
        ie(ID_PDUSESSION_TYPE, wrap(&PDUSessionType::ipv4))?,
        ie(ID_QOS_FLOW_SETUP_REQUEST_LIST, wrap(&flows))?,
    ];
    let transfer = PDUSessionResourceSetupRequestTransfer::new(PDUSessionResourceSetupRequestTransferProtocolIEs(ies));
    wrap(&transfer).map(|a| a.as_bytes().to_vec()).map_err(|e| format!("encoding: {e}"))
}

/// PDU Session Resource Release Command Transfer: a normal release.
pub fn release_command_transfer() -> Result<Vec<u8>, String> {
    let transfer = PDUSessionResourceReleaseCommandTransfer::new(Cause::nas(CauseNas::normal_release), None);
    wrap(&transfer).map(|a| a.as_bytes().to_vec()).map_err(|e| format!("encoding: {e}"))
}

/// The gNB's downlink tunnel endpoint, from a PDU Session Resource Setup
/// Response Transfer.
pub fn setup_response_transfer(bytes: &[u8]) -> Result<(Ipv4Addr, u32), String> {
    let transfer: PDUSessionResourceSetupResponseTransfer =
        open(&Any::new(bytes.to_vec())).map_err(|e| format!("setup response transfer: {e}"))?;
    let UPTransportLayerInformation::gTPTunnel(tunnel) =
        transfer.d_lqos_flow_per_tnlinformation.u_ptransport_layer_information
    else {
        return Err("no GTP tunnel".into());
    };
    // IPv4 (32 bits), possibly followed by IPv6 (160 bits in all).
    let bits = &tunnel.transport_layer_address.0;
    if bits.len() != 32 && bits.len() != 160 {
        return Err(format!("transport layer address of {} bits: IPv4 only", bits.len()));
    }
    let octet = |i: usize| bits[i * 8..i * 8 + 8].iter().fold(0u8, |acc, b| acc << 1 | u8::from(*b));
    let addr = Ipv4Addr::new(octet(0), octet(1), octet(2), octet(3));
    Ok((addr, u32::from_be_bytes(*tunnel.g_tp_teid.0)))
}

/// The cause of a PDU Session Resource Setup Unsuccessful Transfer, for logs.
pub fn unsuccessful_transfer(bytes: &[u8]) -> String {
    match open::<PDUSessionResourceSetupUnsuccessfulTransfer>(&Any::new(bytes.to_vec())) {
        Ok(t) => format!("{:?}", t.cause),
        Err(e) => format!("undecodable unsuccessful transfer: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_transfer_carries_the_upf_tunnel_and_the_default_flow() {
        let setup = Setup {
            upf: Ipv4Addr::new(127, 0, 0, 2),
            teid: 0x1234,
            qfi: 1,
            five_qi: 9,
            arp: 8,
            ambr_bps: (1_000_000, 2_000_000),
        };
        let bytes = setup_request_transfer(&setup).unwrap();
        let t: PDUSessionResourceSetupRequestTransfer = open(&Any::new(bytes)).unwrap();
        let ie = |id: ProtocolIEID| &t.protocol_ies.0.iter().find(|ie| ie.id == id.0).unwrap().value;
        let UPTransportLayerInformation::gTPTunnel(tunnel) = open(ie(ID_UL_NGU_UP_TNLINFORMATION)).unwrap() else {
            panic!()
        };
        assert_eq!(tunnel.transport_layer_address.0.as_raw_slice(), [127, 0, 0, 2]);
        assert_eq!(*tunnel.g_tp_teid.0, [0, 0, 0x12, 0x34]);
        assert_eq!(open::<PDUSessionType>(ie(ID_PDUSESSION_TYPE)).unwrap(), PDUSessionType::ipv4);
        let flows: QosFlowSetupRequestList = open(ie(ID_QOS_FLOW_SETUP_REQUEST_LIST)).unwrap();
        assert_eq!(flows.0[0].qos_flow_identifier.0, Integer::from(1));
        let ambr: PDUSessionAggregateMaximumBitRate = open(ie(ID_PDUSESSION_AGGREGATE_MAXIMUM_BIT_RATE)).unwrap();
        assert_eq!(ambr.p_dusession_aggregate_maximum_bit_rate_dl.0, Integer::from(2_000_000), "downlink first");
    }

    #[test]
    fn the_response_transfer_gives_the_gnb_tunnel() {
        let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
            TransportLayerAddress(BitString::from_slice(&[127, 0, 0, 1])),
            GTPTEID(FixedOctetString::new([0, 0, 0, 7])),
            None,
        ));
        let flows =
            AssociatedQosFlowList(vec![AssociatedQosFlowItem::new(QosFlowIdentifier(Integer::from(1)), None, None)]);
        let transfer = PDUSessionResourceSetupResponseTransfer::new(
            QosFlowPerTNLInformation::new(tunnel, flows, None),
            None,
            None,
            None,
            None,
        );
        let bytes = wrap(&transfer).unwrap().as_bytes().to_vec();
        assert_eq!(setup_response_transfer(&bytes), Ok((Ipv4Addr::new(127, 0, 0, 1), 7)));
        assert!(setup_response_transfer(&[0xff]).is_err());
        let failed = PDUSessionResourceSetupUnsuccessfulTransfer::new(
            Cause::radioNetwork(CauseRadioNetwork::unspecified),
            None,
            None,
        );
        let bytes = wrap(&failed).unwrap().as_bytes().to_vec();
        assert!(unsuccessful_transfer(&bytes).contains("unspecified"));
    }
}
