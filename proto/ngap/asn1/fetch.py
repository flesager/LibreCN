#!/usr/bin/env python3
"""Fetch the NGAP ASN.1 modules of a TS 38.413 version from the 3GPP archive.

    proto/ngap/asn1/fetch.py 15.13.0

Downloads 38413-<code>.zip, extracts the text between "-- ASN1START" and
"-- ASN1STOP" from the specification (.doc or .docx), normalises Word
typography to ASCII, and writes one file per module in asn1/<version>/.
Standard library only.
"""

import io
import re
import sys
import urllib.request
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

ARCHIVE = "https://www.3gpp.org/ftp/Specs/archive/38_series/38.413/"
# The site refuses requests without a browser-like user agent.
USER_AGENT = "Mozilla/5.0 (X11; Linux x86_64) librecn-fetch"
MODULES = 6
# Word typography found in the specifications, and its ASCII form.
NORMALISE = {"“": '"', "”": '"', "‘": "'", "’": "'", " ": " ",
             "–": "-", "—": "-", "\x93": '"', "\x94": '"', "\x91": "'", "\x92": "'"}
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def code(version: str) -> str:
    """15.13.0 -> fd0: release letter (15 = f), then one character per number (10 = a)."""
    major, minor, patch = (int(p) for p in version.split("."))
    digit = "0123456789abcdefghijklmnopqrstuvwxyz"
    return chr(ord("f") + major - 15) + digit[minor] + digit[patch]


def text_of_doc(data: bytes) -> str:
    # Word 97 keeps ASCII text as single bytes; paragraphs end with \r.
    return data.decode("latin-1").replace("\r", "\n")


def text_of_docx(data: bytes) -> str:
    root = ET.fromstring(zipfile.ZipFile(io.BytesIO(data)).read("word/document.xml"))
    lines = []
    for p in root.iter(W + "p"):
        parts = []
        for el in p.iter():
            if el.tag == W + "t":
                parts.append(el.text or "")
            elif el.tag == W + "tab":
                parts.append("\t")
            elif el.tag in (W + "br", W + "cr"):
                parts.append("\n")
        lines.append("".join(parts))
    return "\n".join(lines)


def main() -> None:
    if len(sys.argv) != 2 or not re.fullmatch(r"\d+\.\d+\.\d+", sys.argv[1]):
        sys.exit(__doc__)
    version = sys.argv[1]
    name = f"38413-{code(version)}.zip"
    request = urllib.request.Request(ARCHIVE + name, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=120) as response:
        archive = zipfile.ZipFile(io.BytesIO(response.read()))
    spec = next(n for n in archive.namelist() if n.endswith((".doc", ".docx")))
    data = archive.read(spec)
    text = text_of_docx(data) if spec.endswith(".docx") else text_of_doc(data)

    blocks = re.findall(r"-- ASN1START(.*?)-- ASN1STOP", text, re.S)
    if len(blocks) != MODULES:
        sys.exit(f"{spec}: expected {MODULES} ASN.1 blocks, found {len(blocks)}")
    out = Path(__file__).resolve().parent / version
    out.mkdir(exist_ok=True)
    for block in blocks:
        for typographic, ascii_ in NORMALISE.items():
            block = block.replace(typographic, ascii_)
        odd = sorted({c for c in block if not (c in "\n\t" or " " <= c <= "~")})
        if odd:
            sys.exit(f"{spec}: unexpected characters {[hex(ord(c)) for c in odd]}")
        module = re.search(r"^\s*([A-Z][A-Za-z0-9-]*)\s*\{", block, re.M).group(1)
        body = "\n".join(line.rstrip() for line in block.strip("\n").splitlines())
        header = f"-- {module}: 3GPP TS 38.413 V{version} ({name}), extracted by fetch.py\n\n"
        (out / f"{module}.asn").write_text(header + body + "\n")
        print(f"{out.name}/{module}.asn: {body.count(chr(10)) + 1} lines")


if __name__ == "__main__":
    main()
