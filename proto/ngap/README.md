# NGAP codec (`librecn-ngap`)

NGAP, the NG Application Protocol of N2 (3GPP TS 38.413), ASN.1 Aligned PER.
Generated from the 3GPP ASN.1 with `rasn-compiler`, run by `rasn`.

## Pipeline

| Step | Tool | Output (checked in) |
|---|---|---|
| 1. fetch the spec, extract and normalise the ASN.1 | `asn1/fetch.py 15.13.0` | `asn1/15.13.0/*.asn` (6 modules) |
| 2. generate the bindings, procedure table and deep check | `cargo run --release --manifest-path codegen/Cargo.toml -- 15.13.0` | `src/generated.rs`, `src/procedures.rs` |
| 3. import test vectors | `tests/vectors/import_free5gc.py` | `tests/vectors/free5gc/*.bin`, `SOURCE` |

- **3GPP archive**: `https://www.3gpp.org/ftp/Specs/archive/38_series/38.413/`. Version letters: f=15, g=16, h=17, i=18, j=19; for example `38413-fd0.zip` = v15.13.0.
- **Generator**: `codegen/` is a standalone crate, so `rasn-compiler` never enters normal builds.
- **Changing release**: steps 1 and 2 with the new version, then `VERSION` in `src/lib.rs`.

## Use

Open types stay undecoded (`Any`) until asked for: a PDU is decoded level by level, and only the IEs actually used.

```rust
let pdu = librecn_ngap::decode(bytes)?;                        // NGAP-PDU
let code = librecn_ngap::procedure_code(&pdu);                 // which procedure
let request: NGSetupRequest = librecn_ngap::open(&message.value)?;
let node: GlobalRANNodeID = librecn_ngap::open(&ie.value)?;    // one IE
```

- **Unknown IEs** (a later release) stay opaque and are skipped, as TS 38.413 §10 requires.
- **`check(pdu)`** decodes and re-encodes every level it knows and compares the bytes.

## Tests

- **Vectors**: 358 real PDUs from free5GC (Apache-2.0, see `tests/vectors/free5gc/SOURCE`).
  - `ci*`: recorded by free5GC's CI; valid.
  - `h*`: fuzzer findings, malformed on purpose.
- **Byte-identical round trip**:
  - every valid PDU at the top level;
  - every level for the 16 procedures present (1,774 values).
- **Robustness**: every truncation and random bit flips of every vector must never make `decode` or `check` panic.
- **Regressions**: the extension value of an enumeration (`ci0832`), and an unknown IE.

## `rasn` 0.29 APER bugs, fixed in our fork

Found by the deep round trip on the free5GC vectors. The expected encodings
were cross-checked with an independent implementation, asn1tools 0.169
(aligned PER).

| # | Bug (`src/per/`) | NGAP symptom |
|---|---|---|
| 1 | decode: a bit-field length of a `SEQUENCE OF` is followed by octet alignment; the components follow at once (X.691 §11.9.3.3, §20.6) | `BroadcastPLMNList` (4-bit length) misread |
| 2 | encode: a length of range 256 or more (one or two octets) is not octet-aligned first (§11.5.7.2–3); decode does not align for range 256 | an IE container of 4 IEs written `00 02 00` instead of `00 00 04` |
| 3 | both: a constrained length needs `ub` < 64K (§11.9.3.3), not `range` ≤ 64K | `UE-associatedLogicalNG-connectionList` (`SIZE(1..65536)`) misread |
| 4 | decode: a fixed `BIT STRING` over 16 bits is not octet-aligned (§16.10) | `SecurityKey` of `SecurityContext` read 3 bits early; also `NRCellIdentity` (36 bits) when it does not follow an aligned field |

`rasn`'s own test `issue501` asserted bug 1 for APER; the fork corrects it
(ten `BOOLEAN`s are `9f fc` in APER as in UPER).

**The fork**: branch `librecn-aper-fixes` from tag `rasn-v0.29.0`, with one
commit, kept as `rasn/0001-*.patch` in this directory. It is used through
`[patch.crates-io]` in the workspace `Cargo.toml`. It passes `rasn`'s 566 tests
and 5 regression tests for the bugs above. To recreate it:

```
git clone https://github.com/librasn/rasn && cd rasn
git checkout -b librecn-aper-fixes rasn-v0.29.0
git am <LibreCN>/proto/ngap/rasn/0001-*.patch
```

Not reported upstream yet (see `CLAUDE.md`).

## Compared with hampi (`asn1-codecs` 0.7.2), same Rel-15 ASN.1 and vectors

| | rasn (fork) | hampi |
|---|---|---|
| Valid PDUs round-tripped at every level | 353 / 353 (fork) | 352 / 353 |
| Panics on 38,000+ damaged inputs | 0 | 0 |
| Extension value of an enumeration (`ci0832`) | `notAvailable` | **decoded as `emergency`** (value 0) |
| Unknown IE from a later release | skipped | **whole message refused** ("Key 999 Not Found") |
| Decoding | level by level, only what is used | always everything; `log::trace!` calls in the decoders |

hampi's two failures are structural (typed open types with no room for the unknown), and both break interoperability with newer gNBs. `rasn`'s were local codec bugs, each now fixed with a regression test.
