//! Load on the whole core: every EO (AMF, AUSF, UDM, SMF, UPF, log, alarms)
//! in one pool pinned to CORES cores (2 on).
//!
//! `cargo run --release -p librecn --example load -- CORES GNBS UES_PER_GNB WINDOW [up[=MBPS]] [size=BYTES]`
//! (`scripts/load.sh` runs the series given in `README.md`).
//!
//! - Control plane: test gNBs on SCTP register `UES_PER_GNB` UEs each, with
//!   real 5G AKA and NAS security, then set up their PDU session. A
//!   registration is timed from its Initial UE Message to the PDU Session
//!   Resource Setup Request that brings the UE's session (8 NGAP messages,
//!   3 round trips through AUSF and UDM, one through SMF and UPF). Each gNB
//!   is a thread pinned to its own core (8 on), keeping `WINDOW`
//!   registrations in flight.
//! - With `up`, the user plane too: the UPF gets N3 (127.0.0.2) and N6 (a
//!   datagram socket pair standing for the TUN device). Over the sessions
//!   just set up, packets of `BYTES` (1400) go both ways, as fast as the UPF takes
//!   them, or `MBPS` Mbit/s each way (uplink senders on cores 12 and 13,
//!   downlink on 14, receivers on 6 and 7): first alone, then while the
//!   gNBs register as many UEs again.
//! - Memory: the process's resident memory (and its peak) and heap in use,
//!   empty, once LibreCN is ready, then after each batch of registrations;
//!   and what each EO preallocated.

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixDatagram;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Barrier, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use librecn_amf::Amf;
use librecn_amf::sctp::{NGAP_PPID, Received, SctpSocket};
use librecn_ausf::Ausf;
use librecn_ngap::*;
use librecn_oam::{AlarmManager, LogSink};
use librecn_pcf::Pcf;
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_security::milenage::Milenage;
use librecn_security::{Direction, kdf, nea2, nia2};
use librecn_smf::Smf;
use librecn_udm::Udm;
use librecn_upf::Upf;
use rasn::types::{BitString, FixedBitString, FixedOctetString, Integer};

const K: [u8; 16] = [0x46, 0x5b, 0x5c, 0xe8, 0xb1, 0x99, 0xb4, 0x9f, 0xaa, 0x5f, 0x0a, 0x2e, 0xe2, 0x38, 0xa6, 0xbc];
const OPC: [u8; 16] = [0xe8, 0xed, 0x28, 0x9d, 0xeb, 0xa9, 0x52, 0xe4, 0x28, 0x3b, 0x54, 0xe8, 0x8e, 0x61, 0x83, 0xca];
/// free5GC's test gNB (vector ci0000) serves PLMN 208/93.
const SNN: &str = "5G:mnc093.mcc208.3gppnetwork.org";
const PLMN: [u8; 3] = [0x02, 0xf8, 0x39];
/// First MSIN (10 digits): subscriber `i` is imsi-20893 followed by FIRST + i.
const FIRST: u64 = 1;
/// The UPF's N3 address; the test gNBs' GTP-U is on 127.0.0.1.
const N3: std::net::Ipv4Addr = std::net::Ipv4Addr::new(127, 0, 0, 2);
const GTPU_PORT: u16 = 2152;
/// Default IP packet size of the user-plane load.
const PACKET: usize = 1400;
/// How long the user plane is measured alone.
const UP_ALONE: Duration = Duration::from_secs(2);

fn pin(core: usize) {
    // SAFETY: an all-zero cpu_set_t is a valid empty set.
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: core < CPU_SETSIZE; the set outlives the call.
    unsafe {
        libc::CPU_SET(core, &mut set);
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
    }
}

fn location() -> Any {
    let plmn = PLMNIdentity(FixedOctetString::new(PLMN));
    let tai = TAI::new(plmn.clone(), TAC(FixedOctetString::new([0, 0, 1])), None);
    let cgi = NRCGI::new(plmn, NRCellIdentity(FixedBitString::ZERO), None);
    wrap(&UserLocationInformation::userLocationInformationNR(UserLocationInformationNR::new(cgi, tai, None, None)))
        .unwrap()
}

fn initial_ue_message(ran_id: u32, nas: &[u8], location: &Any) -> Vec<u8> {
    use AnonymousInitialUEMessageProtocolIEsCriticality::{ignore, reject};
    let ies = vec![
        AnonymousInitialUEMessageProtocolIEs::new(ID_RAN_UE_NGAP_ID.0, reject, wrap(&RANUENGAPID(ran_id)).unwrap()),
        AnonymousInitialUEMessageProtocolIEs::new(ID_NAS_PDU.0, reject, wrap(&NASPDU(nas.to_vec().into())).unwrap()),
        AnonymousInitialUEMessageProtocolIEs::new(ID_USER_LOCATION_INFORMATION.0, reject, location.clone()),
        AnonymousInitialUEMessageProtocolIEs::new(
            ID_RRCESTABLISHMENT_CAUSE.0,
            ignore,
            wrap(&RRCEstablishmentCause::mo_Signalling).unwrap(),
        ),
    ];
    let message = wrap(&InitialUEMessage::new(InitialUEMessageProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_INITIAL_UEMESSAGE, Criticality::ignore, message)))
        .unwrap()
}

fn uplink_nas(amf_id: u64, ran_id: u32, nas: &[u8], location: &Any) -> Vec<u8> {
    let ie = |id: ProtocolIEID, value: Any| AnonymousUplinkNASTransportProtocolIEs::new(id, Criticality::reject, value);
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)).unwrap()),
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)).unwrap()),
        ie(ID_NAS_PDU, wrap(&NASPDU(nas.to_vec().into())).unwrap()),
        ie(ID_USER_LOCATION_INFORMATION, location.clone()),
    ];
    let message = wrap(&UplinkNASTransport::new(UplinkNASTransportProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_UPLINK_NASTRANSPORT, Criticality::ignore, message)))
        .unwrap()
}

fn successful<T: rasn::Encode>(code: ProcedureCode, message: &T) -> Vec<u8> {
    encode(&NGAPPDU::successfulOutcome(SuccessfulOutcome::new(code, Criticality::reject, wrap(message).unwrap())))
        .unwrap()
}

fn context_setup_response(amf_id: u64, ran_id: u32) -> Vec<u8> {
    use AnonymousInitialContextSetupResponseProtocolIEsCriticality::ignore;
    let ies = vec![
        AnonymousInitialContextSetupResponseProtocolIEs::new(
            ID_AMF_UE_NGAP_ID.0,
            ignore,
            wrap(&AMFUENGAPID(amf_id)).unwrap(),
        ),
        AnonymousInitialContextSetupResponseProtocolIEs::new(
            ID_RAN_UE_NGAP_ID.0,
            ignore,
            wrap(&RANUENGAPID(ran_id)).unwrap(),
        ),
    ];
    successful(ID_INITIAL_CONTEXT_SETUP, &InitialContextSetupResponse::new(InitialContextSetupResponseProtocolIEs(ies)))
}

fn session_setup_response(amf_id: u64, ran_id: u32) -> Vec<u8> {
    use AnonymousPDUSessionResourceSetupResponseProtocolIEsCriticality::ignore;
    let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
        TransportLayerAddress(BitString::from_slice(&[127, 0, 0, 1])),
        GTPTEID(FixedOctetString::new(ran_id.to_be_bytes())),
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
    let item = PDUSessionResourceSetupItemSURes::new(
        PDUSessionID(1),
        wrap(&transfer).unwrap().as_bytes().to_vec().into(),
        None,
    );
    let ies = vec![
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_AMF_UE_NGAP_ID.0,
            ignore,
            wrap(&AMFUENGAPID(amf_id)).unwrap(),
        ),
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_RAN_UE_NGAP_ID.0,
            ignore,
            wrap(&RANUENGAPID(ran_id)).unwrap(),
        ),
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_PDUSESSION_RESOURCE_SETUP_LIST_SURES.0,
            ignore,
            wrap(&PDUSessionResourceSetupListSURes(vec![item])).unwrap(),
        ),
    ];
    successful(
        ID_PDUSESSION_RESOURCE_SETUP,
        &PDUSessionResourceSetupResponse::new(PDUSessionResourceSetupResponseProtocolIEs(ies)),
    )
}

/// What the gNB got for a UE.
enum Down {
    Nas(Vec<u8>),
    ContextSetup,
    /// The session's NAS (the ciphered accept) and the SMF's N2 transfer.
    SessionSetup(Vec<u8>, Vec<u8>),
}

/// The UE ids and what came, from a downlink NGAP PDU; none for Overload
/// Start and Stop (the AMF's `max_ues` is the number of UEs: it reaches 90 %).
fn downlink(bytes: &[u8]) -> Option<(u64, u32, Down)> {
    let NGAPPDU::initiatingMessage(m) = decode(bytes).unwrap() else { panic!("not an initiating message") };
    fn ids(ies: impl Iterator<Item = (u16, Any)>) -> (u64, u32, Option<Any>) {
        let (mut amf_id, mut ran_id, mut payload) = (0, 0, None);
        for (id, value) in ies {
            match id {
                10 => amf_id = open::<AMFUENGAPID>(&value).unwrap().0,
                85 => ran_id = open::<RANUENGAPID>(&value).unwrap().0,
                // The NAS PDU, or the list of sessions to set up.
                38 | 74 => payload = Some(value),
                _ => {}
            }
        }
        (amf_id, ran_id, payload)
    }
    match m.procedure_code {
        ID_DOWNLINK_NASTRANSPORT => {
            let msg: DownlinkNASTransport = open(&m.value).unwrap();
            let (a, r, nas) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            Some((a, r, Down::Nas(open::<NASPDU>(&nas.unwrap()).unwrap().0.to_vec())))
        }
        ID_INITIAL_CONTEXT_SETUP => {
            let msg: InitialContextSetupRequest = open(&m.value).unwrap();
            let (a, r, _) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            Some((a, r, Down::ContextSetup))
        }
        ID_PDUSESSION_RESOURCE_SETUP => {
            let msg: PDUSessionResourceSetupRequest = open(&m.value).unwrap();
            let (a, r, list) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            let item = open::<PDUSessionResourceSetupListSUReq>(&list.unwrap()).unwrap().0.remove(0);
            let nas = item.p_dusession_nas_pdu.unwrap().0.to_vec();
            Some((a, r, Down::SessionSetup(nas, item.p_dusession_resource_setup_request_transfer.to_vec())))
        }
        ID_OVERLOAD_START | ID_OVERLOAD_STOP => None,
        ref other => panic!("unexpected procedure {other:?}"),
    }
}

/// A UE being registered.
struct Ue {
    started: Instant,
    /// Its subscriber index.
    sub: u64,
    k_enc: [u8; 16],
    k_int: [u8; 16],
    ul: u32,
}

impl Ue {
    fn protect(&mut self, header: u8, plain: &[u8]) -> Vec<u8> {
        let mut pdu = [&[0x7e, header, 0, 0, 0, 0, self.ul as u8][..], plain].concat();
        nea2(&self.k_enc, self.ul, 1, Direction::Uplink, &mut pdu[7..]);
        let mac = nia2(&self.k_int, self.ul, 1, Direction::Uplink, &pdu[6..]);
        pdu[2..6].copy_from_slice(&mac);
        self.ul += 1;
        pdu
    }

    /// The session set up: the UE's address, read from the ciphered PDU
    /// Session Establishment Accept, and the UPF's uplink TEID.
    fn session(&self, nas: &[u8], transfer: &[u8]) -> Session {
        // DL NAS Transport (TS 24.501 §8.7.2): payload container type, then
        // the 5GSM message (§8.3.2): its type, QoS rules, AMBR, PDU address.
        let mut plain = nas[7..].to_vec();
        nea2(&self.k_enc, u32::from(nas[6]), 1, Direction::Downlink, &mut plain);
        let sm = &plain[6..];
        let ambr = 7 + usize::from(u16::from_be_bytes([sm[5], sm[6]]));
        let address = ambr + 1 + usize::from(sm[ambr]);
        assert_eq!(sm[address], 0x29, "the PDU address after the AMBR");
        let ue_ip = sm[address + 3..address + 7].try_into().unwrap();
        let t: PDUSessionResourceSetupRequestTransfer = open(&Any::new(transfer.to_vec())).unwrap();
        let tunnel = &t.protocol_ies.0.iter().find(|ie| ie.id == ID_UL_NGU_UP_TNLINFORMATION.0).unwrap().value;
        let UPTransportLayerInformation::gTPTunnel(tunnel) = open(tunnel).unwrap() else { panic!("no GTP tunnel") };
        Session { ue_ip, ul_teid: u32::from_be_bytes(*tunnel.g_tp_teid.0) }
    }
}

/// A UE's session as the user plane needs it.
#[derive(Clone, Copy)]
struct Session {
    ue_ip: [u8; 4],
    ul_teid: u32,
}

/// The Registration Request of subscriber `index`: SUCI (null scheme), all algorithms.
fn registration_request(index: u64) -> Vec<u8> {
    let msin = format!("{:010}", FIRST + index);
    let d: Vec<u8> = msin.bytes().map(|b| b - b'0').collect();
    let bcd: Vec<u8> = d.chunks(2).map(|p| p[1] << 4 | p[0]).collect();
    [&[0x7e, 0, 0x41, 0x79, 0, 13, 0x01][..], &PLMN, &[0, 0, 0, 0], &bcd, &[0x2e, 4, 0xf0, 0xf0, 0xf0, 0xf0]].concat()
}

/// A test gNB: its SCTP association, after NG Setup.
struct Gnb {
    socket: SctpSocket,
    core: usize,
    location: Any,
    milenage: Milenage,
    /// RAN-UE-NGAP-IDs used so far.
    ran_ids: u32,
}

/// What a round of registrations gave: the latencies and the sessions.
type Round = (Vec<Duration>, Vec<Session>);

impl Gnb {
    fn connect(addr: SocketAddr, core: usize, setup: &[u8]) -> Gnb {
        pin(core);
        let socket = SctpSocket::connect(addr).unwrap();
        socket.send(0, 0, NGAP_PPID, setup).unwrap();
        let mut buf = vec![0u8; 65_536];
        loop {
            if let Some(Received::Data { data, .. }) = socket.recv(&mut buf).unwrap() {
                assert!(matches!(decode(data).unwrap(), NGAPPDU::successfulOutcome(_)), "NG Setup refused");
                break;
            }
        }
        Gnb { socket, core, location: location(), milenage: Milenage::new(&K, &OPC), ran_ids: 0 }
    }

    /// Registers subscribers `first` to `first + count`, `window` at a time.
    fn register(&mut self, first: u64, count: usize, window: usize) -> Round {
        let (socket, location, core) = (&self.socket, &self.location, self.core);
        let mut ues: HashMap<u32, Ue> = HashMap::with_capacity(window);
        let mut latencies = Vec::with_capacity(count);
        let mut sessions = Vec::with_capacity(count);
        let mut failures = 0usize;
        let mut next = 0usize;
        let ran_base = self.ran_ids;
        let start = |next: &mut usize, ues: &mut HashMap<u32, Ue>| {
            let ran_id = ran_base + *next as u32 + 1;
            let sub = first + *next as u64;
            let nas = registration_request(sub);
            socket.send(0, 1 + (ran_id % 8) as u16, NGAP_PPID, &initial_ue_message(ran_id, &nas, location)).unwrap();
            ues.insert(ran_id, Ue { started: Instant::now(), sub, k_enc: [0; 16], k_int: [0; 16], ul: 0 });
            *next += 1;
        };
        while next < count.min(window) {
            start(&mut next, &mut ues);
        }
        let mut buf = vec![0u8; 65_536];
        let fd = std::os::fd::AsRawFd::as_raw_fd(&std::os::fd::AsFd::as_fd(socket));
        while latencies.len() < count {
            let mut pollfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
            // SAFETY: one valid pollfd.
            if unsafe { libc::poll(&mut pollfd, 1, 5000) } == 0 {
                let mut stuck: Vec<_> = ues.iter().map(|(r, u)| (*r, u.ul)).collect();
                stuck.sort();
                panic!(
                    "gNB {core}: stalled at {}/{count}, {} UE(s) waiting (RAN id, uplink count): {:?}",
                    latencies.len(),
                    stuck.len(),
                    &stuck[..stuck.len().min(8)]
                );
            }
            let Some(Received::Data { data, .. }) = socket.recv(&mut buf).unwrap() else { continue };
            let Some((amf_id, ran_id, down)) = downlink(data) else { continue };
            let ue = ues.get_mut(&ran_id).expect("a UE of this gNB");
            let replies: Vec<Vec<u8>> = match down {
                Down::Nas(nas) if nas[1] == 0 && nas[2] == 0x56 => {
                    // Authentication Request: RES*, and the NAS keys for later.
                    let rand: [u8; 16] = nas[8..24].try_into().unwrap();
                    let v = self.milenage.f2345(&rand);
                    let sqn_xor_ak: [u8; 6] = nas[26..32].try_into().unwrap();
                    let kseaf = kdf::kseaf(&kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak), SNN);
                    let imsi = format!("20893{:010}", FIRST + ue.sub);
                    (ue.k_enc, ue.k_int) = kdf::nas_keys(&kdf::kamf(&kseaf, &imsi, &[0, 0]), 2, 2);
                    let res_star = kdf::res_star(&v.ck, &v.ik, SNN, &rand, &v.res);
                    vec![uplink_nas(amf_id, ran_id, &[&[0x7e, 0, 0x57, 0x2d, 0x10][..], &res_star].concat(), location)]
                }
                Down::Nas(nas) if nas[1] == 3 => {
                    // Security Mode Command: Complete with the whole request, slices included.
                    let whole = [&registration_request(ue.sub)[..], &[0x2f, 2, 1, 1]].concat();
                    let complete = [&[0x7e, 0, 0x5e, 0x71][..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
                    vec![uplink_nas(amf_id, ran_id, &ue.protect(4, &complete), location)]
                }
                Down::ContextSetup => {
                    let complete = ue.protect(2, &[0x7e, 0, 0x43]);
                    let n1 = [0x2e, 1, 1, 0xc1, 0xff, 0xff, 0x91, 0xa1];
                    let session = [&[0x7e, 0, 0x67, 0x01, 0, n1.len() as u8][..], &n1, &[0x12, 1, 0x81]].concat();
                    let session = ue.protect(2, &session);
                    vec![
                        context_setup_response(amf_id, ran_id),
                        uplink_nas(amf_id, ran_id, &complete, location),
                        uplink_nas(amf_id, ran_id, &session, location),
                    ]
                }
                Down::SessionSetup(nas, transfer) => {
                    latencies.push(ue.started.elapsed());
                    sessions.push(ue.session(&nas, &transfer));
                    ues.remove(&ran_id);
                    if next < count {
                        start(&mut next, &mut ues);
                    }
                    vec![session_setup_response(amf_id, ran_id)]
                }
                // A refusal (overload): counted, the UE given up.
                Down::Nas(_) => {
                    failures += 1;
                    latencies.push(Duration::MAX);
                    ues.remove(&ran_id);
                    if next < count {
                        start(&mut next, &mut ues);
                    }
                    vec![]
                }
            };
            // The gNB's own stream for the UE (it has 10 outbound streams).
            for reply in replies {
                socket.send(0, 1 + (ran_id % 8) as u16, NGAP_PPID, &reply).unwrap();
            }
        }
        self.ran_ids += count as u32;
        if failures > 0 {
            eprintln!("gNB {core}: {failures} registration(s) refused");
        }
        latencies.retain(|l| *l != Duration::MAX);
        (latencies, sessions)
    }
}

/// User-plane traffic over `sessions`, both ways, until dropped: packets
/// counted where they leave the UPF (N6 uplink, N3 downlink).
struct Traffic {
    stop: Arc<AtomicBool>,
    uplink: Arc<AtomicU64>,
    downlink: Arc<AtomicU64>,
    threads: Vec<JoinHandle<()>>,
}

/// Sends with `send(k)` for k = 0, 1, ..., `per_ms` each millisecond
/// (unlimited if 0), until `stop`.
fn paced(stop: &AtomicBool, per_ms: usize, mut send: impl FnMut(usize)) {
    let start = Instant::now();
    for k in 0.. {
        if k % 1024 == 0 && stop.load(Relaxed) {
            return;
        }
        if per_ms > 0 && k % per_ms == 0 {
            let due = start + Duration::from_millis((k / per_ms) as u64);
            if stop.load(Relaxed) {
                return;
            }
            std::thread::sleep(due.saturating_duration_since(Instant::now()));
        }
        send(k);
    }
}

impl Traffic {
    /// `mbps` each way, 0 for as fast as the UPF takes them, in packets of `size` bytes.
    fn start(sessions: Arc<Vec<Session>>, dn: &UnixDatagram, mbps: usize, size: usize) -> Traffic {
        // Packets per millisecond for each sender: two uplink, one downlink.
        let per_ms = mbps * 1000 / (size * 8);
        let (up_per_ms, down_per_ms) = (per_ms.div_ceil(2), per_ms);
        let stop = Arc::new(AtomicBool::new(false));
        let (uplink, downlink) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let gnb = UdpSocket::bind(("127.0.0.1", GTPU_PORT)).expect("127.0.0.1:2152 free");
        gnb.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        dn.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        let mut threads = Vec::new();
        // Uplink: G-PDUs from the gNB side, each session in turn.
        for core in [12, 13] {
            let (stop, sessions) = (stop.clone(), sessions.clone());
            threads.push(std::thread::spawn(move || {
                pin(core);
                let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
                let mut g_pdu = [&[0x34, 0xff][..], &((8 + size) as u16).to_be_bytes(), &[0; 4]].concat();
                g_pdu.extend_from_slice(&[0, 0, 0, 0x85, 1, 0x10, 1, 0]);
                g_pdu.extend_from_slice(&ipv4([10, 45, 0, 1], size));
                paced(&stop, up_per_ms, |k| {
                    let s = sessions[(2 * k + core) % sessions.len()];
                    g_pdu[4..8].copy_from_slice(&s.ul_teid.to_be_bytes());
                    g_pdu[16 + 12..16 + 16].copy_from_slice(&s.ue_ip);
                    let _ = socket.send_to(&g_pdu, (N3, GTPU_PORT));
                });
            }));
        }
        // Downlink: IP packets on N6 to each UE in turn.
        let sender = dn.try_clone().unwrap();
        let (stop_down, sessions_down) = (stop.clone(), sessions.clone());
        threads.push(std::thread::spawn(move || {
            pin(14);
            let mut packet = ipv4([10, 45, 0, 1], size);
            paced(&stop_down, down_per_ms, |k| {
                packet[16..20].copy_from_slice(&sessions_down[k % sessions_down.len()].ue_ip);
                let _ = sender.send(&packet);
            });
        }));
        // What leaves the UPF.
        let receiver = dn.try_clone().unwrap();
        let (stop_n6, count_n6) = (stop.clone(), uplink.clone());
        threads.push(std::thread::spawn(move || {
            pin(6);
            let mut buf = [0u8; 2048];
            while !stop_n6.load(Relaxed) {
                if receiver.recv(&mut buf).is_ok() {
                    count_n6.fetch_add(1, Relaxed);
                }
            }
        }));
        let (stop_n3, count_n3) = (stop.clone(), downlink.clone());
        threads.push(std::thread::spawn(move || {
            pin(7);
            let mut buf = [0u8; 2048];
            while !stop_n3.load(Relaxed) {
                if gnb.recv(&mut buf).is_ok() {
                    count_n3.fetch_add(1, Relaxed);
                }
            }
        }));
        Traffic { stop, uplink, downlink, threads }
    }

    /// Packets out of the UPF so far: (uplink, downlink).
    fn counts(&self) -> (u64, u64) {
        (self.uplink.load(Relaxed), self.downlink.load(Relaxed))
    }
}

impl Drop for Traffic {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        self.threads.drain(..).for_each(|t| t.join().unwrap());
    }
}

/// An IPv4 packet of `size` bytes from `src`; its destination left zero.
fn ipv4(src: [u8; 4], size: usize) -> Vec<u8> {
    let len = size as u16;
    let mut p = vec![0x45, 0, (len >> 8) as u8, len as u8, 0, 0, 0, 0, 64, 17, 0, 0];
    p.extend_from_slice(&src);
    p.resize(size, 0);
    p
}

/// Busy time of the pool's workers so far.
fn busy(runtime: &Runtime) -> Duration {
    runtime.worker_stats().iter().map(|w| w.busy).sum()
}

/// "N/TOTAL registered, R registrations/s, latency p50 / p99" of a round
/// that took `elapsed`.
fn registrations(round: &mut [Duration], total: usize, elapsed: Duration) -> String {
    round.sort();
    let p = |q: f64| round[((round.len() - 1) as f64 * q) as usize].as_secs_f64() * 1e3;
    format!(
        "{}/{total} registered, {:.0} registrations/s, latency p50 {:.2} ms, p99 {:.2} ms, max {:.2} ms",
        round.len(),
        round.len() as f64 / elapsed.as_secs_f64(),
        p(0.5),
        p(0.99),
        p(1.0)
    )
}

/// "uplink / downlink" rates between two counts `elapsed` apart, of
/// packets of `size` bytes.
fn user_plane(before: (u64, u64), after: (u64, u64), elapsed: Duration, size: usize) -> String {
    let rate = |n: u64| {
        let kpps = n as f64 / elapsed.as_secs_f64() / 1e3;
        format!("{kpps:.0} kpps, {:.2} Gbit/s", kpps * (size * 8) as f64 / 1e6)
    };
    format!("uplink {}, downlink {}", rate(after.0 - before.0), rate(after.1 - before.1))
}

/// Heap bytes in use (allocated and not freed), in MiB: what the
/// allocator keeps besides is not counted.
fn heap() -> f64 {
    // SAFETY: mallinfo2 only reads the allocator's statistics.
    let info = unsafe { libc::mallinfo2() };
    (info.uordblks + info.hblkhd) as f64 / 1048576.0
}

/// The process's resident memory and its peak so far, in MiB.
fn memory() -> (f64, f64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kib = |key: &str| {
        let line = status.lines().find(|l| l.starts_with(key)).unwrap_or_default();
        line.split_whitespace().nth(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0)
    };
    (kib("VmRSS:") / 1024.0, kib("VmHWM:") / 1024.0)
}

fn main() {
    let empty = (memory().0, heap());
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // `up`: user plane as fast as it goes (0); `up=MBPS`: that rate each way.
    let (mut up, mut size) = (None, PACKET);
    while let Some(option) = args.pop_if(|a| a.starts_with("up") || a.starts_with("size=")) {
        match option.split_once('=') {
            None => up = Some(0),
            Some(("up", mbps)) => up = Some(mbps.parse().expect("up=MBPS")),
            Some((_, bytes)) => size = bytes.parse().expect("size=BYTES"),
        }
    }
    assert!((28..=1500).contains(&size), "size: 28 to 1500 bytes");
    let args: Vec<usize> = args.iter().map(|a| a.parse().expect("numbers")).collect();
    let [cores, gnbs, per_gnb, window] = args[..] else {
        panic!("usage: load CORES GNBS UES_PER_GNB WINDOW [up[=MBPS]] [size=BYTES]");
    };
    let rounds = if up.is_some() { 2 } else { 1 };
    let total = gnbs * per_gnb;
    let pool: Vec<usize> = (2..2 + cores).collect();
    let mut config = RuntimeConfig::new(vec![PoolConfig::pinned("core", &pool)]);
    config.max_eos = 16;
    let mut runtime = Runtime::new(config).unwrap();
    librecn_common::log::set_default_level(librecn_common::log::Level::Error);
    let log = runtime.add(LogSink::config().pool("core"), LogSink::stdout()).unwrap();
    let alarm = runtime.add(AlarmManager::config().pool("core"), AlarmManager::new()).unwrap();
    let subscribers = serde_json::json!({ "subscribers": [{
        "supi": format!("imsi-20893{FIRST:010}"), "count": rounds * total,
        "k": "465B5CE8B199B49FAA5F0A2EE238A6BC", "opc": "E8ED289DEBA952E4283B54E88E6183CA" }] });
    let (cfg, udm) = Udm::from_params("udm", &subscribers).unwrap();
    let mut nfs = vec![("udm", runtime.add(cfg.pool("core"), udm).unwrap())];
    let (cfg, ausf) = Ausf::from_params("ausf", &serde_json::json!({ "max_contexts": 4096 })).unwrap();
    nfs.push(("ausf", runtime.add(cfg.pool("core"), ausf).unwrap()));
    let (cfg, pcf) = Pcf::from_params("pcf", &serde_json::Value::Null).unwrap();
    nfs.push(("pcf", runtime.add(cfg.pool("core"), pcf).unwrap()));
    let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
    nfs.push(("smf", runtime.add(cfg.pool("core"), smf).unwrap()));
    let upf_params = serde_json::json!({ "max_sessions": rounds * total, "n3": "127.0.0.2" });
    let (cfg, mut upf) = Upf::from_params("upf", &upf_params).unwrap();
    let mut dn = None;
    if up.is_some() {
        let (ours, upf_end) = UnixDatagram::pair().unwrap();
        upf_end.set_nonblocking(true).unwrap();
        upf = upf.with_n6(std::fs::File::from(OwnedFd::from(upf_end)));
        dn = Some(ours);
    }
    nfs.push(("upf", runtime.add(cfg.pool("core"), upf).unwrap()));
    let params =
        serde_json::json!({ "n2": "127.0.0.1:0", "plmn": { "mcc": "208", "mnc": "93" }, "max_ues": rounds * total });
    let (cfg, amf) = Amf::from_params("amf-1", &params).unwrap();
    let bound = amf.bound_addr();
    nfs.push(("amf", runtime.add(cfg.pool("core"), amf).unwrap()));
    let addr = loop {
        if let Some(addr) = *bound.lock().unwrap() {
            break addr;
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    let ready = (memory().0, heap());
    let setup =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../proto/ngap/tests/vectors/free5gc/ci0000.bin")).unwrap();
    let setup = Arc::new(setup);
    // Each round starts when the gNBs and this thread meet at the barrier.
    let barrier = Arc::new(Barrier::new(gnbs + 1));
    let (results, done) = mpsc::channel::<Round>();
    let threads: Vec<_> = (0..gnbs)
        .map(|g| {
            let (setup, barrier, results) = (setup.clone(), barrier.clone(), results.clone());
            std::thread::spawn(move || {
                let mut gnb = Gnb::connect(addr, 8 + g, &setup);
                for round in 0..rounds {
                    barrier.wait();
                    let first = (round * total + g * per_gnb) as u64;
                    results.send(gnb.register(first, per_gnb, window)).unwrap();
                }
            })
        })
        .collect();
    let round = |busy_before: Duration| {
        let started = Instant::now();
        barrier.wait();
        let (mut latencies, mut sessions) = (Vec::new(), Vec::new());
        for _ in 0..gnbs {
            let (l, s) = done.recv().unwrap();
            latencies.extend(l);
            sessions.extend(s);
        }
        let elapsed = started.elapsed();
        let load = (busy(&runtime) - busy_before).as_secs_f64() / elapsed.as_secs_f64() / cores as f64 * 100.0;
        (latencies, sessions, elapsed, load)
    };
    let head = format!("cores {cores}, {gnbs} gNB(s) x {per_gnb} UEs, window {window}");
    let (mut latencies, sessions, elapsed, load) = round(busy(&runtime));
    println!("{head}: control plane alone: {}, pool busy {load:.0} %", registrations(&mut latencies, total, elapsed));
    let mut registered = vec![(total, memory().0, heap())];
    if let (Some(dn), Some(mbps)) = (&dn, up) {
        let traffic = Traffic::start(Arc::new(sessions), dn, mbps, size);
        std::thread::sleep(Duration::from_millis(500)); // warm up
        let (before, busy_before) = (traffic.counts(), busy(&runtime));
        std::thread::sleep(UP_ALONE);
        let load = (busy(&runtime) - busy_before).as_secs_f64() / UP_ALONE.as_secs_f64() / cores as f64 * 100.0;
        println!(
            "{head}: user plane alone ({total} sessions, {size} B): {}, pool busy {load:.0} %",
            user_plane(before, traffic.counts(), UP_ALONE, size)
        );
        let before = traffic.counts();
        let (mut latencies, _, elapsed, load) = round(busy(&runtime));
        println!(
            "{head}: both: {}; {}; pool busy {load:.0} %",
            registrations(&mut latencies, total, elapsed),
            user_plane(before, traffic.counts(), elapsed, size)
        );
        registered.push((2 * total, memory().0, heap()));
    }
    for t in threads {
        t.join().unwrap();
    }
    // What each EO preallocated (mailbox, buffers, timers, I/O, recorder).
    nfs.extend([("log", log), ("alarm", alarm)]);
    let reserved: Vec<(&str, f64)> = nfs
        .iter()
        .filter_map(|(name, id)| Some((*name, runtime.stats(*id)?.creation.bytes_reserved as f64 / 1048576.0)))
        .collect();
    let after: Vec<String> =
        registered.iter().map(|(n, m, h)| format!("after {n} UEs {m:.1} MiB (heap {h:.1})")).collect();
    println!(
        "{head}: memory: empty process {:.1} MiB (heap {:.1}), LibreCN ready {:.1} MiB (heap {:.1}), {}, \
         peak {:.1} MiB; preallocated by the EOs {:.1} MiB ({})",
        empty.0,
        empty.1,
        ready.0,
        ready.1,
        after.join(", "),
        memory().1,
        reserved.iter().map(|(_, m)| m).sum::<f64>(),
        reserved.iter().map(|(n, m)| format!("{n} {m:.2}")).collect::<Vec<_>>().join(", ")
    );
    if std::env::var_os("LOAD_STATS").is_some() {
        for (name, id) in &nfs {
            let Some(s) = runtime.stats(*id) else { continue };
            let p = &s.processing;
            println!(
                "  {name}: {} messages, mean {:.1} µs, max {:.0} µs, busy {:.0} ms, data queue peak {}",
                p.count,
                p.mean().unwrap_or_default().as_secs_f64() * 1e6,
                p.max.as_secs_f64() * 1e6,
                p.total.as_secs_f64() * 1e3,
                s.data_lane.high_water
            );
        }
    }
    runtime.shutdown(Duration::from_secs(5));
}
