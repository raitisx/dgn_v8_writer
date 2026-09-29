#!/usr/bin/env python3
"""Inspection tool used for EXP-0001 (needs `pip install olefile`).

  v8dump.py entries FILE               CFB entries with sizes and flags
  v8dump.py objects FILE STORAGE       objects of every $N page in STORAGE
  v8dump.py hex FILE STORAGE ID[,ID]   hex dump of objects by element ID
  v8dump.py stream FILE PATH [ZOFF]    hex dump of a stream, inflated from ZOFF

STORAGE is for example "Dgn-Md/#000000/Dgn^G" or "Dgn^Nm".
"""

import struct
import sys
import zlib

import olefile


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


def main(argv):
    command, path = argv[1], argv[2]
    ole = olefile.OleFileIO(path)
    if command == "entries":
        for entry in ole.listdir(streams=True, storages=True):
            name = "/".join(entry)
            kind = "stream" if ole.get_type(name) == olefile.STGTY_STREAM else "storage"
            size = ole.get_size(name) if kind == "stream" else 0
            print(f"{kind:7} {size:8} {name!r}")
    elif command == "objects":
        for _, page in pages(ole, argv[3]):
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
    elif command == "hex":
        wanted = {int(x) for x in argv[4].split(",")}
        for _, page in pages(ole, argv[3]):
            _, body = parse_page(ole.openstream(page).read())
            for _, _, obj in objects(body):
                if len(obj) >= 0x18 and struct.unpack_from("<Q", obj, 0x10)[0] in wanted:
                    print(f"-- id={struct.unpack_from('<Q', obj, 0x10)[0]} type={obj[0]} bytes={len(obj)}")
                    hexdump(obj)
    elif command == "stream":
        raw = ole.openstream(argv[3]).read()
        if len(argv) > 4:
            offset = int(argv[4], 0)
            print("prefix:")
            hexdump(raw[:offset])
            raw = inflate_at(raw, offset)
            print(f"inflated ({len(raw)} bytes):")
        hexdump(raw)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
