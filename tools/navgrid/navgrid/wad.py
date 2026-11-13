# SPDX-License-Identifier: AGPL-3.0-only
#
# Portions ported from TheKillerey/MapgeoAddon wad_tool.py (parse_wad,
# read_entry_data, the ENTRY_* codes) at commit
# 8d2f8bd9f57d064c47a5b9c5c7e6e55597412d76:
#   Copyright (c) 2026 TheKillerey. MIT License; the full notice is in
#   tools/navgrid/NOTICE.md. Rewritten without bpy, for v3 only, on an mmap,
#   with the xxhash package instead of its pure-Python xxhash64.
"""Read a League WAD v3.x archive and find the Summoner's Rift navgrid in it.

Layout (little-endian):

    "RW" u8 major u8 minor                     (major 3 accepted)
    256 bytes signature, 8 bytes checksum, u32 entry_count   -> TOC at byte 272
    TOC entry, 32 bytes each:
        u64 path_hash   xxhash64(seed 0) of the lowercased path, '\\' -> '/'
        u32 offset      absolute offset of the stored data
        u32 compressed_size
        u32 uncompressed_size
        u8  type        low nibble = storage type, high nibble = subchunk count
        u8  duplicate
        u16 first subchunk index
        u64 checksum    (first 8 bytes of the data's SHA-256; not checked)

Storage types: 0 raw, 1 gzip, 2 satellite (refused), 3 zstd, 4 zstd-chunked
(zstd frames back to back; a sub-chunk that did not compress is stored raw
between them, so raw runs between frames are copied as they are).
"""

from __future__ import annotations

import gzip
import hashlib
import io
import mmap
import struct
from dataclasses import dataclass, field
from pathlib import Path

import xxhash

TYPE_RAW = 0
TYPE_GZIP = 1
TYPE_SATELLITE = 2
TYPE_ZSTD = 3
TYPE_ZSTD_CHUNKED = 4
TYPE_NAMES = {0: "raw", 1: "gzip", 2: "satellite", 3: "zstd", 4: "zstd-chunked"}

ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
HEADER_V3 = 272
TOC_ENTRY = struct.Struct("<QIIIBBHQ")  # 32 bytes

# Candidate navgrid paths for Summoner's Rift, in the order they are tried.
# Source: CommunityDragon map11.bin.json, patch 16.20.8248524 (fetched
# 2026-10-20): MapSkins/Default/mNavigationMesh names AIPath_SRX_2 (live SR,
# 21 skins); AIPath (14 older event skins) and AIPath_SRX (one test skin) are
# also named. MapgeoAddon verified "aipath_srx"; the 2024 converter repo uses
# "_3". AIPath_NPE_1 (tutorial) is deliberately not a candidate.
# The hash is xxhash64 (seed 0) of the lowercased path; test_wad checks them.
CANDIDATE_PATHS: tuple[tuple[str, int], ...] = (
    ("assets/maps/navgrid/map11/aipath_srx_2.aimesh_ngrid", 0xE08D20957D2E2A3A),
    ("assets/maps/navgrid/map11/aipath_srx_3.aimesh_ngrid", 0x9D67B1C0B806E23E),
    ("assets/maps/navgrid/map11/aipath_srx.aimesh_ngrid", 0xB7B2B65287ED2102),
    ("assets/maps/navgrid/map11/aipath.aimesh_ngrid", 0x59C8E0541C0960CE),
)

# Content sniffing bounds (v7 header, 50-unit cells, 200..400 cells a side).
SNIFF_MIN_COUNT = 200
SNIFF_MAX_COUNT = 400
SNIFF_CELL_SIZE = 50.0
SNIFF_MAX_BYTES = 256 * 1024 * 1024


class WadError(ValueError):
    pass


def path_hash(path: str) -> int:
    """xxhash64 (seed 0) of the WAD path, lowercased with '\\' -> '/'."""
    return xxhash.xxh64_intdigest(path.lower().replace("\\", "/").encode("utf-8"))


@dataclass
class WadEntry:
    path_hash: int
    offset: int
    compressed_size: int
    uncompressed_size: int
    type: int
    subchunks: int
    duplicate: bool

    @property
    def type_name(self) -> str:
        return TYPE_NAMES.get(self.type, f"unknown-{self.type}")


@dataclass
class Found:
    """The navgrid entry and how it was found."""

    data: bytes
    entry: WadEntry
    method: str          # "candidate-path" or "content-sniff"
    path: str | None     # the candidate path, or None when sniffed
    sniffed: list[str] = field(default_factory=list)


class WadArchive:
    """A WAD read from one open file (memory-mapped)."""

    def __init__(self, buf, path: Path | None = None):
        self.buf = buf
        self.path = path
        if len(buf) < 4 or bytes(buf[0:2]) != b"RW":
            raise WadError("not a WAD file (magic is not 'RW')")
        self.major, self.minor = buf[2], buf[3]
        if self.major != 3:
            raise WadError(f"unsupported WAD version {self.major}.{self.minor} (v3.x expected)")
        if len(buf) < HEADER_V3:
            raise WadError("WAD truncated inside its header")
        (count,) = struct.unpack_from("<I", buf, 268)
        end = HEADER_V3 + count * TOC_ENTRY.size
        if end > len(buf):
            raise WadError(f"WAD truncated inside its table of contents ({count} entries)")
        self.entries: list[WadEntry] = []
        for i in range(count):
            h, off, cs, us, t, dup, _first, _chk = TOC_ENTRY.unpack_from(buf, HEADER_V3 + i * TOC_ENTRY.size)
            self.entries.append(WadEntry(h, off, cs, us, t & 0x0F, t >> 4, bool(dup)))
        self.by_hash = {e.path_hash: e for e in self.entries}

    @property
    def version(self) -> str:
        return f"{self.major}.{self.minor}"

    def raw(self, e: WadEntry) -> bytes:
        if e.offset + e.compressed_size > len(self.buf):
            raise WadError(f"entry {e.path_hash:016x} lies past the end of the file")
        return bytes(self.buf[e.offset:e.offset + e.compressed_size])

    def read(self, e: WadEntry) -> bytes:
        """The entry's uncompressed bytes."""
        data = self.raw(e)
        if e.type == TYPE_RAW:
            out = data
        elif e.type == TYPE_GZIP:
            out = gzip.decompress(data)
        elif e.type == TYPE_ZSTD:
            import zstandard

            out = zstandard.ZstdDecompressor().decompress(data, max_output_size=e.uncompressed_size)
        elif e.type == TYPE_ZSTD_CHUNKED:
            out = _zstd_chunked(data, e.uncompressed_size)
        elif e.type == TYPE_SATELLITE:
            raise WadError("satellite entries point outside the WAD; not supported")
        else:
            raise WadError(f"unknown entry type {e.type}")
        if len(out) != e.uncompressed_size:
            raise WadError(
                f"entry {e.path_hash:016x}: {len(out)} bytes after decoding, TOC says {e.uncompressed_size}"
            )
        return out

    def head(self, e: WadEntry, n: int = 64) -> bytes:
        """The first n uncompressed bytes, without decoding the whole entry."""
        data = self.raw(e)
        if e.type == TYPE_RAW:
            return data[:n]
        if e.type == TYPE_GZIP:
            with gzip.GzipFile(fileobj=io.BytesIO(data)) as g:
                return g.read(n)
        if e.type in (TYPE_ZSTD, TYPE_ZSTD_CHUNKED):
            if e.type == TYPE_ZSTD_CHUNKED and not data.startswith(ZSTD_MAGIC):
                return _zstd_chunked(data, e.uncompressed_size, limit=n)[:n]
            import zstandard

            with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(data)) as r:
                return r.read(n)
        return b""

    # -- finding the navgrid ------------------------------------------------

    def find_navgrid(self, candidates=CANDIDATE_PATHS) -> Found:
        """Candidate paths first, then content sniffing. Raises WadError if none."""
        for p, h in candidates:
            e = self.by_hash.get(h)
            if e is not None:
                return Found(self.read(e), e, "candidate-path", p)
        sniffed = []
        for e in sorted(self.entries, key=lambda x: x.offset):
            if e.type == TYPE_SATELLITE or not (2_000_000 <= e.uncompressed_size <= SNIFF_MAX_BYTES):
                continue
            try:
                if sniff_ngrid_header(self.head(e, 64)):
                    sniffed.append(f"{e.path_hash:016x}")
            except Exception:  # noqa: BLE001 - a broken neighbour must not stop the search
                continue
        if sniffed:
            e = next(x for x in self.entries if f"{x.path_hash:016x}" == sniffed[0])
            return Found(self.read(e), e, "content-sniff", None, sniffed)
        raise WadError("no navgrid found: no candidate path and no entry with a v7 navgrid header")


def sniff_ngrid_header(head: bytes) -> bool:
    """A v7 navgrid header with 50-unit cells and 200..400 cells per side."""
    if len(head) < 39 or head[0] != 7:
        return False
    cell, cx, cz = struct.unpack_from("<fii", head, 27)
    return (
        abs(cell - SNIFF_CELL_SIZE) < 1e-3
        and SNIFF_MIN_COUNT <= cx <= SNIFF_MAX_COUNT
        and SNIFF_MIN_COUNT <= cz <= SNIFF_MAX_COUNT
    )


def _zstd_chunked(data: bytes, size: int, limit: int | None = None) -> bytes:
    """Concatenated zstd frames, with raw runs between them copied through."""
    import zstandard

    out = bytearray()
    pos = 0
    target = size if limit is None else min(size, limit)
    while pos < len(data) and len(out) < target:
        if data.startswith(ZSTD_MAGIC, pos):
            dobj = zstandard.ZstdDecompressor().decompressobj()
            out += dobj.decompress(data[pos:])
            used = len(data) - pos - len(dobj.unused_data)
            if used <= 0:
                raise WadError("zstd-chunked entry: a frame did not decode")
            pos += used
        else:
            nxt = data.find(ZSTD_MAGIC, pos)
            nxt = len(data) if nxt < 0 else nxt
            out += data[pos:nxt]
            pos = nxt
    return bytes(out)


def open_wad(path: Path):
    """Open the WAD once: (archive, sha256 hex, open file). Close the file after use."""
    f = open(path, "rb")
    try:
        buf = mmap.mmap(f.fileno(), 0, access=mmap.ACCESS_READ)
    except ValueError:  # empty file
        f.close()
        raise WadError("empty file")
    sha = hashlib.sha256()
    view = memoryview(buf)
    step = 16 * 1024 * 1024
    for i in range(0, len(buf), step):
        sha.update(view[i:i + step])
    view.release()
    try:
        arc = WadArchive(buf, path)
    except Exception:
        buf.close()
        f.close()
        raise
    return arc, sha.hexdigest(), (buf, f)


def close_wad(handle) -> None:
    buf, f = handle
    buf.close()
    f.close()


# -- writing (tests only: synthetic archives) ---------------------------------


def write_wad(entries: list[tuple[int, bytes, int]]) -> bytes:
    """A WAD v3.4 holding (path_hash, payload, type) entries. Synthetic use only.

    zstd-chunked payloads are split into 3 chunks; the middle one is stored raw.
    """
    import zstandard

    blobs = []
    for h, payload, t in entries:
        if t == TYPE_RAW:
            stored, sub = payload, 0
        elif t == TYPE_GZIP:
            stored, sub = gzip.compress(payload, mtime=0), 0
        elif t == TYPE_ZSTD:
            stored, sub = zstandard.ZstdCompressor().compress(payload), 0
        elif t == TYPE_ZSTD_CHUNKED:
            n = len(payload)
            a, b = n // 3, 2 * n // 3
            c = zstandard.ZstdCompressor()
            stored = c.compress(payload[:a]) + payload[a:b] + c.compress(payload[b:])
            sub = 3
        else:
            raise ValueError(t)
        blobs.append((h, stored, len(payload), t | (sub << 4)))
    out = bytearray(b"RW" + bytes([3, 4]) + bytes(256) + bytes(8) + struct.pack("<I", len(blobs)))
    off = HEADER_V3 + TOC_ENTRY.size * len(blobs)
    toc, data = bytearray(), bytearray()
    for h, stored, usize, tb in blobs:
        chk = struct.unpack("<Q", hashlib.sha256(stored).digest()[:8])[0]
        toc += TOC_ENTRY.pack(h, off + len(data), len(stored), usize, tb, 0, 0, chk)
        data += stored
    return bytes(out + toc + data)
