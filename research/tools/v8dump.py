#!/usr/bin/env python3
"""Inspection tool for DGN V8 files (needs `pip install olefile`).

  v8dump.py summary FILE               size, SHA-256, saving application, save time
  v8dump.py entries FILE               CFB entries with sizes and flags
  v8dump.py pages FILE                 every $N page header; model header word
  v8dump.py objects FILE STORAGE       objects of every $N page in STORAGE
  v8dump.py hex FILE STORAGE ID[,ID]   hex dump of objects by element ID
  v8dump.py linkages FILE STORAGE ID[,ID]   linkages of objects by element ID
  v8dump.py tables FILE                table headers (96) and entries (95)
  v8dump.py stream FILE PATH [ZOFF]    hex dump of a stream, inflated from ZOFF
  v8dump.py diff FILE_A FILE_B         streams that differ, after inflating

STORAGE is for example "Dgn-Md/#000000/Dgn^G" or "Dgn^Nm".

Files to inspect are untrusted data: run this with `python3 -I` from a
directory other than the one holding the file.
"""

import hashlib
import signal
import struct
import sys
import zlib

import olefile

STRING_LINKAGE_ID = 0x56D2
LIBRARY_BIT = 0x0080_0000


def inflate_at(data, offset):
    d = zlib.decompressobj()
    out = d.decompress(data[offset:])
    if not d.eof or d.unused_data:
        raise ValueError("incomplete or trailing zlib data")
    return out


def pages(ole, storage):
    found = []
    for entry in ole.listdir(streams=True, storages=False):
        path = "/".join(entry)
        if path.startswith(storage + "/$"):
            found.append((int(entry[-1][1:]), path))
    return sorted(found)


def all_pages(ole):
    found = []
    for entry in ole.listdir(streams=True, storages=False):
        if entry[-1].startswith("$") and entry[-1][1:].isdigit():
            found.append("/".join(entry))
    return sorted(found)


def parse_page(raw):
    header = struct.unpack_from("<4I", raw, 0)
    return header, (inflate_at(raw, 16) if len(raw) > 16 else b"")


def objects(body):
    offset, found = 0, []
    while offset < len(body):
        prefix = struct.unpack_from("<I", body, offset)[0]
        start = offset + 4
        words = struct.unpack_from("<I", body, start + 4)[0]
        found.append((offset, prefix, body[start:start + 2 * words]))
        offset = start + 2 * words
    if offset != len(body):
        raise ValueError("object framing does not end at page end")
    return found


def hexdump(data):
    for i in range(0, len(data), 16):
        print(f"    {i:04x}: " + " ".join(f"{b:02x}" for b in data[i:i + 16]))


def decode_payload(data):
    """Text and string-linkage payload forms (FN-T02, FN-T05, FN-T07, FN-L03)."""
    if data.startswith(b"\xff\xfe\x01\x00"):
        text = data[4:].decode("cp1252", "replace")
    elif data[:2] in (b"\xff\xfe", b"\xff\xfd"):
        text = data[2:len(data) - (len(data) % 2)].decode("utf-16-le", "replace")
    else:
        text = data.decode("cp1252", "replace")
    return text.rstrip("\x00")


def linkages(obj):
    """(linkage id, bytes) pairs after the attribute offset (FN-L01)."""
    found, at = [], 2 * struct.unpack_from("<I", obj, 8)[0]
    while at + 4 <= len(obj):
        length = 2 * (obj[at] + 1)
        if obj[at + 1] & 0x10 == 0 or at + length > len(obj):
            break
        found.append((struct.unpack_from("<H", obj, at + 2)[0], obj[at:at + length]))
        at += length
    return found


def string_property(obj, property_id):
    for linkage_id, data in linkages(obj):
        if linkage_id == STRING_LINKAGE_ID and len(data) >= 12:
            pid, length = struct.unpack_from("<II", data, 4)
            if pid == property_id:
                return decode_payload(data[12:12 + length])
    return None


def element_id(obj):
    return struct.unpack_from("<Q", obj, 0x10)[0] if len(obj) >= 0x18 else None


def objects_with_ids(ole, storage, wanted):
    for _, page in pages(ole, storage):
        _, body = parse_page(ole.openstream(page).read())
        for _, _, obj in objects(body):
            if element_id(obj) in wanted:
                yield obj


def logical(data):
    """Stream content with any zlib part inflated (offsets 0, 0x10, 0x14)."""
    for offset in (0, 0x10, 0x14):
        if len(data) > offset + 2 and data[offset] == 0x78:
            try:
                return data[:offset] + inflate_at(data, offset), offset
            except (zlib.error, ValueError):
                pass
    return data, None


def cmd_summary(path):
    data = open(path, "rb").read()
    ole = olefile.OleFileIO(path)
    meta = ole.get_metadata()
    app = meta.creating_application
    if isinstance(app, bytes):
        app = app.decode("latin-1", "replace")
    app = (app or "").split("\x00")[0][:80]
    print(f"file      {path}")
    print(f"bytes     {len(data)}")
    print(f"sha256    {hashlib.sha256(data).hexdigest()}")
    print(f"saved by  {app}")
    print(f"saved at  {meta.last_saved_time}")


def cmd_entries(ole):
    for entry in ole.listdir(streams=True, storages=True):
        name = "/".join(entry)
        kind = "stream" if ole.get_type(name) == olefile.STGTY_STREAM else "storage"
        size = ole.get_size(name) if kind == "stream" else 0
        print(f"{kind:7} {size:8} {name!r}")


def cmd_pages(ole):
    for page in all_pages(ole):
        raw = ole.openstream(page).read()
        header = struct.unpack_from("<4I", raw) if len(raw) >= 16 else None
        print(f"{page:36} bytes={len(raw):7} count,version,page,population={header}")
    for entry in ole.listdir(streams=True, storages=False):
        if entry[-1] != "Dgn~Mh":
            continue
        body = inflate_at(ole.openstream(entry).read(), 0)
        for at in range(0, len(body) - 12, 2):
            words = struct.unpack_from("<I", body, at + 4)[0]
            if body[at] == 66 and at + 2 * words == len(body):
                word = struct.unpack_from("<I", body, at)[0]
                print(f"{'/'.join(entry)}: u32 before header = {struct.unpack_from('<I', body, 0)[0]},"
                      f" type-66 word {word:#010x} at {at:#x}")


def cmd_objects(ole, storage):
    for _, page in pages(ole, storage):
        header, body = parse_page(ole.openstream(page).read())
        print(f"== {page}: count={header[0]} version={header[1]} page={header[2]} population={header[3]}")
        for offset, prefix, obj in objects(body):
            word, words, attr = struct.unpack_from("<3I", obj, 0)
            line = f"  @{offset:6} prefix={prefix:#x} type={word & 0xff:3} role={word >> 24:#04x} words={words} attr={attr}"
            if len(obj) >= 0x68:
                level, eid, stamp = struct.unpack_from("<IQd", obj, 0x0c)
                gg, props, flags, style, weight, color = struct.unpack_from("<6I", obj, 0x20)
                line += (f" level={level} id={eid} time={stamp:.0f} gg={gg} props={props:#x}"
                         f" flags={flags:#x} style={style} weight={weight} color={color}")
            print(line)


def cmd_hex(ole, storage, ids):
    for obj in objects_with_ids(ole, storage, ids):
        print(f"-- id={element_id(obj)} type={obj[0]} bytes={len(obj)}")
        hexdump(obj)


def cmd_linkages(ole, storage, ids):
    for obj in objects_with_ids(ole, storage, ids):
        word, _, attr = struct.unpack_from("<3I", obj, 0)
        print(f"-- id={element_id(obj)} type={word & 0xff} role={word >> 24:#x}"
              f" primary={2 * attr} total={len(obj)}")
        for linkage_id, data in linkages(obj):
            print(f"   {linkage_id:#06x} ({len(data)} bytes): {data.hex(' ')}")


def cmd_tables(ole):
    """Table objects in every object page; kind is stored at 0x0c (FN-V01)."""
    for page in all_pages(ole):
        if page.rsplit("/", 2)[-2].endswith("A"):
            continue  # auxiliary storages hold other records (FN-P04)
        raw = ole.openstream(page).read()
        if len(raw) <= 16:
            continue
        _, body = parse_page(raw)
        for index, (_, _, obj) in enumerate(objects(body)):
            if obj[0] not in (95, 96) or len(obj) < 0x24:
                continue
            word = struct.unpack_from("<I", obj, 0)[0]
            kind, value = struct.unpack_from("<I", obj, 0x0c)[0], struct.unpack_from("<I", obj, 0x20)[0]
            what = "header" if obj[0] == 96 else "entry"
            copy = " library" if word & LIBRARY_BIT else ""
            name = string_property(obj, 1)
            label = "count" if obj[0] == 96 else "id"
            text = f" name={name!r}" if name is not None else ""
            print(f"{page:24} #{index:<4} {what:6} kind={kind:<3} {label}={value:<6}"
                  f" element={element_id(obj)}{copy}{text}")


def cmd_stream(ole, path, offset=None):
    raw = ole.openstream(path).read()
    if offset is not None:
        print("prefix:")
        hexdump(raw[:offset])
        raw = inflate_at(raw, offset)
        print(f"inflated ({len(raw)} bytes):")
    hexdump(raw)


def cmd_diff(path_a, path_b):
    a, b = olefile.OleFileIO(path_a), olefile.OleFileIO(path_b)
    names_a = {"/".join(e) for e in a.listdir()}
    names_b = {"/".join(e) for e in b.listdir()}
    print("only in A:", sorted(names_a - names_b))
    print("only in B:", sorted(names_b - names_a))
    for name in sorted(names_a & names_b):
        (la, oa), (lb, _) = logical(a.openstream(name).read()), logical(b.openstream(name).read())
        if la != lb:
            diffs = [i for i in range(min(len(la), len(lb))) if la[i] != lb[i]]
            more = "..." if len(diffs) > 12 else ""
            print(f"{name!r:45} zlib@{oa} len {len(la)}->{len(lb)}"
                  f" first differences at {[hex(x) for x in diffs[:12]]}{more}")


def main(argv):
    if len(argv) < 3:
        raise SystemExit(__doc__)
    command, path = argv[1], argv[2]
    if command == "summary":
        return cmd_summary(path)
    if command == "diff" and len(argv) == 4:
        return cmd_diff(path, argv[3])
    ole = olefile.OleFileIO(path)
    if command == "entries":
        cmd_entries(ole)
    elif command == "pages":
        cmd_pages(ole)
    elif command == "objects" and len(argv) == 4:
        cmd_objects(ole, argv[3])
    elif command in ("hex", "linkages") and len(argv) == 5:
        ids = {int(x) for x in argv[4].split(",")}
        (cmd_hex if command == "hex" else cmd_linkages)(ole, argv[3], ids)
    elif command == "tables":
        cmd_tables(ole)
    elif command == "stream" and len(argv) in (4, 5):
        cmd_stream(ole, argv[3], int(argv[4], 0) if len(argv) == 5 else None)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    if hasattr(signal, "SIGPIPE"):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)  # quiet when piped to head
    main(sys.argv)
