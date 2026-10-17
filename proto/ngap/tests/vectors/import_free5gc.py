#!/usr/bin/env python3
"""Import free5GC's NGAP fuzz corpus as test vectors (raw NGAP PDUs).

    proto/ngap/tests/vectors/import_free5gc.py

Downloads github.com/free5gc/ngap (Apache-2.0) at its current main commit,
reads testdata/fuzz/FuzzNGAP/* (Go fuzz corpus files, one []byte literal
each), and writes each distinct message to free5gc/<name>.bin, with the
licence and the commit in free5gc/SOURCE. Standard library only.
"""

import ast
import hashlib
import io
import json
import shutil
import tarfile
import urllib.request
from pathlib import Path

REPO = "free5gc/ngap"
HEADERS = {"User-Agent": "librecn-import"}


def get(url: str) -> bytes:
    with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS), timeout=120) as r:
        return r.read()


def main() -> None:
    commit = json.loads(get(f"https://api.github.com/repos/{REPO}/commits/main"))["sha"]
    archive = tarfile.open(fileobj=io.BytesIO(get(f"https://codeload.github.com/{REPO}/tar.gz/{commit}")))
    out = Path(__file__).resolve().parent / "free5gc"
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir()
    seen, licence = set(), None
    for member in sorted(archive.getmembers(), key=lambda m: m.name):
        parts = member.name.split("/")
        if parts[-1] == "LICENSE" and len(parts) == 2:
            licence = archive.extractfile(member).read().decode()
        if not member.isfile() or parts[-3:-1] != ["fuzz", "FuzzNGAP"]:
            continue
        lines = archive.extractfile(member).read().decode().splitlines()
        if lines[0] != "go test fuzz v1" or len(lines) != 2 or not lines[1].startswith("[]byte("):
            continue
        # Go and Python agree on the escapes used (\xNN, \", \\, \n, ...);
        # literal_eval only parses literals, it runs nothing.
        pdu = ast.literal_eval("b" + lines[1][len("[]byte("):-1])
        digest = hashlib.sha256(pdu).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        name = parts[-1] if parts[-1].startswith("ci") else "h" + parts[-1][:12]
        (out / f"{name}.bin").write_bytes(pdu)
    (out / "SOURCE").write_text(
        f"Raw NGAP PDUs from https://github.com/{REPO} at commit {commit},\n"
        f"testdata/fuzz/FuzzNGAP (Go fuzz corpus), deduplicated by import_free5gc.py.\n"
        f"Licensed under the Apache License 2.0, reproduced below.\n\n{licence}"
    )
    print(f"{len(seen)} distinct PDUs from {REPO}@{commit[:12]}")


if __name__ == "__main__":
    main()
