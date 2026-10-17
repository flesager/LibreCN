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
  - every level for the 16 procedures present (1,756 values).
- **Robustness**: every truncation and random bit flips of every vector must never make `decode` or `check` panic.
- **Regressions**: the extension value of an enumeration (`ci0832`), and an unknown IE.

## `rasn` 0.29 APER issues found

Found by the deep round trip on the free5GC vectors.

1. **`SEQUENCE OF` length, decoding** (`per/de.rs`, `decode_length`): after a bit-field length (range ≤ 255) the decoder octet-aligns. That is right for strings (`decode_string_length` does it conditionally) but wrong for `SEQUENCE OF`, whose components follow at once. `BroadcastPLMNList` (4-bit length sharing an octet with its first item's preamble) then fails to decode.
2. **`SEQUENCE OF` length, encoding**: a two-octet length (range above 256) is written without octet-aligning first. For example `NGSetupRequest` with 4 IEs gives `00 02 00` instead of `00 00 04`.
3. **Upper bound of exactly 64K** (`SIZE(1..65536)`, `UE-associatedLogicalNG-connectionList`): treated as constrained (`range <= 64K`), where X.691 11.9.3.3 requires `ub < 64K` (general length determinant).
4. **Fixed `BIT STRING` over 16 bits, encoding**, after unaligned fields in the same `SEQUENCE`: the alignment padding misses the earlier fields. The 256-bit `SecurityKey` of `SecurityContext` comes out shifted by 3 bits (`10 00 cb 88…` instead of `10 06 5c 43…`). Decoding is right.

**Workarounds:**
- **1–3**: the generator replaces the derived `Encode`/`Decode` of the 400 size-constrained `SEQUENCE OF` wrappers with `aper_sequence_of!` (`src/lib.rs`), which codes the length as the constrained whole number it is, or as a general length from 64K on.
- **4**: no workaround from outside `rasn`. The two affected vectors are listed as known failures in `tests/vectors.rs`.

Not reported upstream yet. Remove the workarounds once `rasn` is fixed.

## Compared with hampi (`asn1-codecs` 0.7.2), same Rel-15 ASN.1 and vectors

| | rasn + workarounds | hampi |
|---|---|---|
| Valid PDUs round-tripped at every level | 351 / 353 (issue 4) | 352 / 353 |
| Panics on 38,000+ damaged inputs | 0 | 0 |
| Extension value of an enumeration (`ci0832`) | `notAvailable` | **decoded as `emergency`** (value 0) |
| Unknown IE from a later release | skipped | **whole message refused** ("Key 999 Not Found") |
| Decoding | level by level, only what is used | always everything; `log::trace!` calls in the decoders |

hampi's two failures are structural (typed open types with no room for the unknown), and both break interoperability with newer gNBs. `rasn`'s are local codec bugs, with a reproduction for each.
