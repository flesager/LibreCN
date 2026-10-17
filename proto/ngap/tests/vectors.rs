//! NGAP codec against real PDUs (free5GC's corpus, see vectors/free5gc/SOURCE).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use librecn_ngap::*;

fn vectors() -> Vec<(String, Vec<u8>)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/free5gc");
    let mut all: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bin"))
        .map(|p| (p.file_stem().unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap()))
        .collect();
    all.sort();
    assert!(all.len() > 300, "vectors missing: run tests/vectors/import_free5gc.py");
    all
}

/// Decodes without letting a panic through; `None` if the decoder panicked.
fn decode_guarded(bytes: &[u8]) -> Option<Result<NGAPPDU, DecodeError>> {
    catch_unwind(AssertUnwindSafe(|| decode(bytes))).ok()
}

/// `ci*`: messages recorded by free5GC's CI, valid. `h*`: inputs its fuzzer
/// found, malformed on purpose.
fn is_valid(name: &str) -> bool {
    name.starts_with("ci")
}

#[test]
fn valid_vectors_decode_and_re_encode_to_the_same_bytes() {
    let (mut checked, mut refused, mut differ) = (0, Vec::new(), Vec::new());
    for (name, bytes) in vectors().into_iter().filter(|(n, _)| is_valid(n)) {
        match decode_guarded(&bytes).unwrap_or_else(|| panic!("{name}: decoder panicked")) {
            Ok(pdu) if encode(&pdu).unwrap() == bytes => checked += 1,
            Ok(_) => differ.push(name),
            Err(e) => refused.push(format!("{name}: {e}")),
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
    assert!(differ.is_empty(), "re-encoding differs for {differ:?}");
    assert!(checked > 300, "{checked}");
}

#[test]
fn valid_vectors_survive_a_round_trip_at_every_level() {
    let (mut values, mut failures, mut procedures) = (0, Vec::new(), std::collections::BTreeSet::new());
    for (name, bytes) in vectors().into_iter().filter(|(n, _)| is_valid(n)) {
        let pdu = decode(&bytes).unwrap();
        let code = procedure_code(&pdu).unwrap();
        procedures.insert(procedure_name(code).unwrap_or("?"));
        match catch_unwind(AssertUnwindSafe(|| check(&pdu))).unwrap_or_else(|_| panic!("{name}: check panicked")) {
            Ok(n) => values += n,
            Err(e) => failures.push(format!("{name} ({}): {e}", procedure_name(code).unwrap_or("?"))),
        }
    }
    assert!(failures.is_empty(), "{} failure(s):\n{}", failures.len(), failures.join("\n"));
    println!("{values} values checked across {} procedures: {procedures:?}", procedures.len());
    assert!(values > 1000, "{values}");
}

#[test]
fn fuzzer_findings_are_refused_without_panic() {
    for (name, bytes) in vectors().into_iter().filter(|(n, _)| !is_valid(n)) {
        let result = decode_guarded(&bytes).unwrap_or_else(|| panic!("{name}: decoder panicked"));
        if let Ok(pdu) = result {
            // Accepting is fine (e.g. non-zero padding bits, which decoders
            // ignore): the value must then survive a re-encoding.
            assert_eq!(decode(&encode(&pdu).unwrap()).unwrap(), pdu, "{name}");
        }
    }
}

#[test]
fn ng_setup_request_is_decoded_step_by_step() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/free5gc/ci0000.bin")).unwrap();
    let NGAPPDU::initiatingMessage(message) = decode(&bytes).unwrap() else { panic!("not an initiating message") };
    assert_eq!(message.procedure_code, ID_NGSETUP);
    let request: NGSetupRequest = open(&message.value).unwrap();

    let ies = &request.protocol_ies.0;
    let ids: Vec<u16> = ies.iter().map(|ie| ie.id).collect();
    assert_eq!(ids, [ID_GLOBAL_RANNODE_ID.0, ID_RANNODE_NAME.0, ID_SUPPORTED_TALIST.0, ID_DEFAULT_PAGING_DRX.0]);
    let value = |id: ProtocolIEID| &ies.iter().find(|ie| ie.id == id.0).unwrap().value;

    let GlobalRANNodeID::globalGNB_ID(gnb) = open(value(ID_GLOBAL_RANNODE_ID)).unwrap() else { panic!("not a gNB") };
    assert_eq!(gnb.p_lmnidentity.0.as_slice(), [0x02, 0xf8, 0x39], "PLMN 208/93");
    let GNBID::gNB_ID(id) = gnb.g_nb_id else { panic!("no gNB id") };
    assert_eq!((id.len(), id.as_raw_slice()), (24, [0x00, 0x01, 0x02].as_slice()), "24-bit gNB id 0x000102");
    let name: RANNodeName = open(value(ID_RANNODE_NAME)).unwrap();
    assert_eq!(name.0.as_bytes(), b"free5gc");
    let tas: SupportedTAList = open(value(ID_SUPPORTED_TALIST)).unwrap();
    assert_eq!(tas.0[0].t_ac.0.as_slice(), [0x00, 0x00, 0x01]);
    let drx: PagingDRX = open(value(ID_DEFAULT_PAGING_DRX)).unwrap();
    assert_eq!(drx, PagingDRX::v128);

    // Built back from the decoded parts: the same bytes.
    let rebuilt =
        NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_NGSETUP, message.criticality, wrap(&request).unwrap()));
    assert_eq!(encode(&rebuilt).unwrap(), bytes);
}

/// Deterministic xorshift, so a failure is reproducible.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn damaged_input_is_refused_never_panics() {
    let mut rng = 0x9e37_79b9_7f4a_7c15;
    let mut tried = 0;
    for (name, bytes) in vectors() {
        let survives =
            |input: &[u8]| catch_unwind(AssertUnwindSafe(|| decode(input).map(|pdu| check(&pdu).is_ok()))).is_ok();
        for len in 0..bytes.len() {
            assert!(survives(&bytes[..len]), "{name}: panic on truncation to {len} bytes");
        }
        for _ in 0..if bytes.is_empty() { 0 } else { 16 } {
            let mut damaged = bytes.clone();
            let at = (xorshift(&mut rng) as usize) % damaged.len();
            damaged[at] ^= 1 << (xorshift(&mut rng) % 8);
            assert!(survives(&damaged), "{name}: panic on {damaged:02x?}");
        }
        tried += bytes.len() + 16;
    }
    assert!(tried > 10_000);
}

fn vector(name: &str) -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/vectors/free5gc/{name}.bin"))).unwrap()
}

#[test]
fn extension_values_of_enumerations_are_kept() {
    // ci0832: Initial UE Message whose RRC establishment cause is the
    // extension value `notAvailable` (0x80), not the root value 0 (emergency).
    let NGAPPDU::initiatingMessage(m) = decode(&vector("ci0832")).unwrap() else { panic!() };
    let message: InitialUEMessage = open(&m.value).unwrap();
    let ie = message.protocol_ies.0.iter().find(|ie| ie.id == ID_RRCESTABLISHMENT_CAUSE.0).unwrap();
    assert_eq!(ie.value.as_bytes(), [0x80]);
    assert_eq!(open::<RRCEstablishmentCause>(&ie.value).unwrap(), RRCEstablishmentCause::notAvailable);
}

#[test]
fn unknown_ies_are_skipped_not_fatal() {
    // ci0000 (NG Setup Request, 4 IEs) plus an IE of a later release: id 999,
    // criticality ignore, one-octet value. TS 38.413 §10: handled by its criticality.
    let original = vector("ci0000");
    assert_eq!(&original[..7], [0x00, 0x15, 0x00, 0x35, 0x00, 0x00, 0x04]);
    let mut bytes = vec![0x00, 0x15, 0x00, 0x35 + 5, 0x00, 0x00, 0x05];
    bytes.extend_from_slice(&original[7..]);
    bytes.extend_from_slice(&[0x03, 0xe7, 0x40, 0x01, 0x00]);
    let pdu = decode(&bytes).unwrap();
    assert_eq!(check(&pdu).unwrap(), 5, "the message and its 4 known IEs");
    assert_eq!(encode(&pdu).unwrap(), bytes);
}
