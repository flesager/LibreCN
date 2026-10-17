//! Control-plane load on the whole core: test gNBs on SCTP register UEs with
//! real 5G AKA and NAS security, then set up their PDU session; LibreCN's
//! NFs (AMF, AUSF, UDM, SMF, UPF) run in one pool pinned to 1 to 4 cores.
//!
//! `cargo run --release -p librecn --example load -- CORES GNBS UES_PER_GNB WINDOW`
//! (`scripts/load.sh` runs the series given in `README.md`).
//!
//! A registration is timed from its Initial UE Message to the PDU Session
//! Resource Setup Request that brings the UE's session (8 NGAP messages,
//! 3 round trips through AUSF and UDM, one through SMF and UPF). Each gNB is a
//! thread pinned to its own core (8 on), keeping `WINDOW` registrations in flight.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use librecn_amf::Amf;
use librecn_amf::sctp::{NGAP_PPID, Received, SctpSocket};
use librecn_ausf::Ausf;
use librecn_ngap::*;
use librecn_oam::{AlarmManager, LogSink};
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
    SessionSetup,
}

/// The UE ids and what came, from a downlink NGAP PDU.
fn downlink(bytes: &[u8]) -> (u64, u32, Down) {
    let NGAPPDU::initiatingMessage(m) = decode(bytes).unwrap() else { panic!("not an initiating message") };
    fn ids(ies: impl Iterator<Item = (u16, Any)>) -> (u64, u32, Option<Vec<u8>>) {
        let (mut amf_id, mut ran_id, mut nas) = (0, 0, None);
        for (id, value) in ies {
            match id {
                10 => amf_id = open::<AMFUENGAPID>(&value).unwrap().0,
                85 => ran_id = open::<RANUENGAPID>(&value).unwrap().0,
                38 => nas = Some(open::<NASPDU>(&value).unwrap().0.to_vec()),
                _ => {}
            }
        }
        (amf_id, ran_id, nas)
    }
    match m.procedure_code {
        ID_DOWNLINK_NASTRANSPORT => {
            let msg: DownlinkNASTransport = open(&m.value).unwrap();
            let (a, r, nas) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            (a, r, Down::Nas(nas.unwrap()))
        }
        ID_INITIAL_CONTEXT_SETUP => {
            let msg: InitialContextSetupRequest = open(&m.value).unwrap();
            let (a, r, _) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            (a, r, Down::ContextSetup)
        }
        ID_PDUSESSION_RESOURCE_SETUP => {
            let msg: PDUSessionResourceSetupRequest = open(&m.value).unwrap();
            let (a, r, _) = ids(msg.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)));
            (a, r, Down::SessionSetup)
        }
        ref other => panic!("unexpected procedure {other:?}"),
    }
}

/// A UE being registered.
struct Ue {
    started: Instant,
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
}

/// The Registration Request of subscriber `index`: SUCI (null scheme), all algorithms.
fn registration_request(index: u64) -> Vec<u8> {
    let msin = format!("{:010}", FIRST + index);
    let d: Vec<u8> = msin.bytes().map(|b| b - b'0').collect();
    let bcd: Vec<u8> = d.chunks(2).map(|p| p[1] << 4 | p[0]).collect();
    [&[0x7e, 0, 0x41, 0x79, 0, 13, 0x01][..], &PLMN, &[0, 0, 0, 0], &bcd, &[0x2e, 4, 0xf0, 0xf0, 0xf0, 0xf0]].concat()
}

/// One gNB: `count` registrations from subscriber `first`, `window` at a time.
fn gnb(addr: SocketAddr, core: usize, first: u64, count: usize, window: usize, setup: &[u8]) -> Vec<Duration> {
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
    let location = location();
    let mut ues: HashMap<u32, Ue> = HashMap::with_capacity(window);
    let mut latencies = Vec::with_capacity(count);
    let mut failures = 0usize;
    let mut next = 0usize;
    let start = |next: &mut usize, ues: &mut HashMap<u32, Ue>| {
        let ran_id = *next as u32 + 1;
        let nas = registration_request(first + *next as u64);
        socket.send(0, 1 + (ran_id % 8) as u16, NGAP_PPID, &initial_ue_message(ran_id, &nas, &location)).unwrap();
        ues.insert(ran_id, Ue { started: Instant::now(), k_enc: [0; 16], k_int: [0; 16], ul: 0 });
        *next += 1;
    };
    while next < count.min(window) {
        start(&mut next, &mut ues);
    }
    let milenage = Milenage::new(&K, &OPC);
    let fd = std::os::fd::AsRawFd::as_raw_fd(&std::os::fd::AsFd::as_fd(&socket));
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
        let (amf_id, ran_id, down) = downlink(data);
        let ue = ues.get_mut(&ran_id).expect("a UE of this gNB");
        let replies: Vec<Vec<u8>> = match down {
            Down::Nas(nas) if nas[1] == 0 && nas[2] == 0x56 => {
                // Authentication Request: RES*, and the NAS keys for later.
                let rand: [u8; 16] = nas[8..24].try_into().unwrap();
                let v = milenage.f2345(&rand);
                let sqn_xor_ak: [u8; 6] = nas[26..32].try_into().unwrap();
                let kseaf = kdf::kseaf(&kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak), SNN);
                let imsi = format!("20893{:010}", FIRST + first + u64::from(ran_id) - 1);
                (ue.k_enc, ue.k_int) = kdf::nas_keys(&kdf::kamf(&kseaf, &imsi, &[0, 0]), 2, 2);
                let res_star = kdf::res_star(&v.ck, &v.ik, SNN, &rand, &v.res);
                vec![uplink_nas(amf_id, ran_id, &[&[0x7e, 0, 0x57, 0x2d, 0x10][..], &res_star].concat(), &location)]
            }
            Down::Nas(nas) if nas[1] == 3 => {
                // Security Mode Command: Complete with the whole request, slices included.
                let whole = [&registration_request(first + u64::from(ran_id) - 1)[..], &[0x2f, 2, 1, 1]].concat();
                let complete = [&[0x7e, 0, 0x5e, 0x71][..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
                vec![uplink_nas(amf_id, ran_id, &ue.protect(4, &complete), &location)]
            }
            Down::ContextSetup => {
                let complete = ue.protect(2, &[0x7e, 0, 0x43]);
                let n1 = [0x2e, 1, 1, 0xc1, 0xff, 0xff, 0x91, 0xa1];
                let session = [&[0x7e, 0, 0x67, 0x01, 0, n1.len() as u8][..], &n1, &[0x12, 1, 0x81]].concat();
                let session = ue.protect(2, &session);
                vec![
                    context_setup_response(amf_id, ran_id),
                    uplink_nas(amf_id, ran_id, &complete, &location),
                    uplink_nas(amf_id, ran_id, &session, &location),
                ]
            }
            Down::SessionSetup => {
                latencies.push(ue.started.elapsed());
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
    if failures > 0 {
        eprintln!("gNB {core}: {failures} registration(s) refused");
    }
    latencies.retain(|l| *l != Duration::MAX);
    latencies
}

fn main() {
    let args: Vec<usize> = std::env::args().skip(1).map(|a| a.parse().expect("numbers")).collect();
    let [cores, gnbs, per_gnb, window] = args[..] else {
        panic!("usage: load CORES GNBS UES_PER_GNB WINDOW");
    };
    let total = gnbs * per_gnb;
    let cp: Vec<usize> = (2..2 + cores).collect();
    let mut config = RuntimeConfig::new(vec![PoolConfig::pinned("cp", &cp), PoolConfig::pinned("oam", &[6])]);
    config.max_eos = 16;
    let mut runtime = Runtime::new(config).unwrap();
    librecn_common::log::set_default_level(librecn_common::log::Level::Error);
    runtime.add(LogSink::config(), LogSink::stdout()).unwrap();
    runtime.add(AlarmManager::config(), AlarmManager::new()).unwrap();
    let subscribers = serde_json::json!({ "subscribers": [{
        "supi": format!("imsi-20893{FIRST:010}"), "count": total,
        "k": "465B5CE8B199B49FAA5F0A2EE238A6BC", "opc": "E8ED289DEBA952E4283B54E88E6183CA" }] });
    let (cfg, udm) = Udm::from_params("udm", &subscribers).unwrap();
    let mut nfs = vec![("udm", runtime.add(cfg.pool("cp"), udm).unwrap())];
    let (cfg, ausf) = Ausf::from_params("ausf", &serde_json::json!({ "max_contexts": 4096 })).unwrap();
    nfs.push(("ausf", runtime.add(cfg.pool("cp"), ausf).unwrap()));
    let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
    nfs.push(("smf", runtime.add(cfg.pool("cp"), smf).unwrap()));
    let (cfg, upf) = Upf::from_params("upf", &serde_json::json!({ "max_sessions": total })).unwrap();
    nfs.push(("upf", runtime.add(cfg.pool("cp"), upf).unwrap()));
    let params = serde_json::json!({ "n2": "127.0.0.1:0", "plmn": { "mcc": "208", "mnc": "93" }, "max_ues": total });
    let (cfg, amf) = Amf::from_params("amf-1", &params).unwrap();
    let bound = amf.bound_addr();
    nfs.push(("amf", runtime.add(cfg.pool("cp"), amf).unwrap()));
    let addr = loop {
        if let Some(addr) = *bound.lock().unwrap() {
            break addr;
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    let setup =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../proto/ngap/tests/vectors/free5gc/ci0000.bin")).unwrap();
    let setup = Arc::new(setup);
    let all = Arc::new(Mutex::new(Vec::with_capacity(total)));
    let started = Instant::now();
    let threads: Vec<_> = (0..gnbs)
        .map(|g| {
            let (setup, all) = (setup.clone(), all.clone());
            std::thread::spawn(move || {
                let latencies = gnb(addr, 8 + g, (g * per_gnb) as u64, per_gnb, window, &setup);
                all.lock().unwrap().extend(latencies);
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let elapsed = started.elapsed();
    let mut latencies = std::mem::take(&mut *all.lock().unwrap());
    let done = latencies.len();
    latencies.sort();
    let p = |q: f64| latencies[((latencies.len() - 1) as f64 * q) as usize].as_secs_f64() * 1e3;
    let busy: Duration = runtime.worker_stats().iter().filter(|w| w.pool == "cp").map(|w| w.busy).sum();
    println!(
        "cores {cores}, {gnbs} gNB(s) x {per_gnb} UEs, window {window}: {done}/{total} registered, \
         {:.0} registrations/s, latency p50 {:.2} ms, p99 {:.2} ms, worker busy {:.0} % of {cores} core(s)",
        done as f64 / elapsed.as_secs_f64(),
        p(0.5),
        p(0.99),
        busy.as_secs_f64() / elapsed.as_secs_f64() / cores as f64 * 100.0
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
