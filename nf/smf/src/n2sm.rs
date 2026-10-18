//! N2 SM information: the NGAP transfers the SMF writes for the gNB and
//! reads from it (TS 38.413 §9.3.4). The only module seeing ASN.1.

use std::net::Ipv4Addr;

use librecn_ngap::*;
use librecn_sbi::policy::{DEFAULT_QFI, PccRule, SessionRule, rule_qfi};
use rasn::types::{BitString, FixedOctetString, Integer};

/// A QoS flow to set up: QFI, 5QI, ARP, and a GBR flow's PCC rule.
fn flow(qfi: u8, five_qi: u8, arp: u8, gbr: Option<&PccRule>) -> QosFlowSetupRequestItem {
    let rate = |bps: u64| BitRate(Integer::from(bps));
    let gbr = gbr.map(|r| {
        GBRQosInformation::new(
            rate(r.mbr.downlink),
            rate(r.mbr.uplink),
            rate(r.gbr.downlink),
            rate(r.gbr.uplink),
            None,
            None,
            None,
            None,
        )
    });
    let qos = QosFlowLevelQosParameters::new(
        QosCharacteristics::nonDynamic5QI(NonDynamic5QIDescriptor::new(
            FiveQI(Integer::from(five_qi)),
            None,
            None,
            None,
            None,
        )),
        AllocationAndRetentionPriority::new(
            PriorityLevelARP(arp),
            PreEmptionCapability::shall_not_trigger_pre_emption,
            PreEmptionVulnerability::not_pre_emptable,
            None,
        ),
        gbr,
        None,
        None,
        None,
    );
    QosFlowSetupRequestItem::new(QosFlowIdentifier(Integer::from(qfi)), qos, None, None)
}

/// PDU Session Resource Setup Request Transfer: the UPF's tunnel endpoint
/// (`upf`, `teid`) and the PCF's `rule`: the session AMBR, the default flow
/// first (UERANSIM's gNB marks every uplink packet with the first flow),
/// then one per PCC rule.
pub fn setup_request_transfer(upf: Ipv4Addr, teid: u32, rule: &SessionRule) -> Result<Vec<u8>, String> {
    use AnonymousPDUSessionResourceSetupRequestTransferProtocolIEsCriticality::reject;
    let ambr = PDUSessionAggregateMaximumBitRate::new(
        BitRate(Integer::from(rule.ambr.downlink)),
        BitRate(Integer::from(rule.ambr.uplink)),
        None,
    );
    let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
        TransportLayerAddress(BitString::from_slice(&upf.octets())),
        GTPTEID(FixedOctetString::new(teid.to_be_bytes())),
        None,
    ));
    let mut flows = vec![flow(DEFAULT_QFI, rule.five_qi, rule.arp, None)];
    for (i, r) in rule.rules().iter().enumerate() {
        flows.push(flow(rule_qfi(i), r.five_qi, r.arp, (!r.gbr.is_zero()).then_some(r)));
    }
    let flows = QosFlowSetupRequestList(flows);
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
    fn the_request_transfer_carries_the_upf_tunnel_and_the_flows() {
        use librecn_sbi::policy::Rates;
        use zerocopy::FromZeros;
        let mut rule = SessionRule::new_zeroed();
        (rule.five_qi, rule.arp, rule.ambr) = (9, 8, Rates { uplink: 1_000_000, downlink: 2_000_000 });
        let mut voice = PccRule::new_zeroed();
        (voice.five_qi, voice.arp) = (1, 2);
        (voice.gbr, voice.mbr) =
            (Rates { uplink: 64_000, downlink: 64_000 }, Rates { uplink: 128_000, downlink: 256_000 });
        let mut video = PccRule::new_zeroed();
        (video.five_qi, video.arp) = (7, 9);
        rule.set_rules(&[voice, video]);
        let bytes = setup_request_transfer(Ipv4Addr::new(127, 0, 0, 2), 0x1234, &rule).unwrap();
        let t: PDUSessionResourceSetupRequestTransfer = open(&Any::new(bytes)).unwrap();
        let ie = |id: ProtocolIEID| &t.protocol_ies.0.iter().find(|ie| ie.id == id.0).unwrap().value;
        let UPTransportLayerInformation::gTPTunnel(tunnel) = open(ie(ID_UL_NGU_UP_TNLINFORMATION)).unwrap() else {
            panic!()
        };
        assert_eq!(tunnel.transport_layer_address.0.as_raw_slice(), [127, 0, 0, 2]);
        assert_eq!(*tunnel.g_tp_teid.0, [0, 0, 0x12, 0x34]);
        assert_eq!(open::<PDUSessionType>(ie(ID_PDUSESSION_TYPE)).unwrap(), PDUSessionType::ipv4);
        let flows: QosFlowSetupRequestList = open(ie(ID_QOS_FLOW_SETUP_REQUEST_LIST)).unwrap();
        let described: Vec<_> = flows
            .0
            .iter()
            .map(|f| {
                let q = &f.qos_flow_level_qos_parameters;
                let QosCharacteristics::nonDynamic5QI(d) = &q.qos_characteristics else { panic!() };
                let mbr_dl = q.g_br_qos_information.as_ref().map(|g| g.maximum_flow_bit_rate_dl.0.clone());
                (
                    f.qos_flow_identifier.0.clone(),
                    d.five_qi.0.clone(),
                    q.allocation_and_retention_priority.priority_level_arp.0,
                    mbr_dl,
                )
            })
            .collect();
        let int = Integer::from;
        assert_eq!(
            described,
            [(int(1), int(9), 8, None), (int(2), int(1), 2, Some(int(256_000))), (int(3), int(7), 9, None)],
            "the default flow first, then one per PCC rule, GBR information for the GBR one"
        );
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
