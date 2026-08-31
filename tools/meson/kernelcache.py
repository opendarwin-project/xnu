"""Mach-O Kernelcache packaging tool for XNU."""

import argparse
import base64
import os
import secrets
import struct
from dataclasses import dataclass

MH_MAGIC_64 = 0xFEEDFACF
MH_EXECUTE = 0x2
CPU_TYPE_ARM64 = 0x0100000C
LC_SEGMENT_64 = 0x19
LC_SYMTAB = 0x2
SEG_CMD_SIZE = 72
SECT_SIZE = 80
PAGE_SIZE = 0x4000  # 16KB
VM_PROT_READ = 1
VM_PROT_WRITE = 2


def pad(n: int, align: int) -> int:
    return (n + align - 1) & ~(align - 1)


@dataclass
class SegmentInfo:
    name: str
    vmaddr: int
    vmsize: int
    fileoff: int
    filesize: int


def parse_macho_header(data: bytes) -> tuple[int, int, int]:
    if len(data) < 32:
        raise ValueError("Data too short for Mach-O header")

    magic, cputype, _, filetype, ncmds, sizeofcmds, _, _ = struct.unpack_from(
        "<IIIIIIII", data, 0
    )
    if magic != MH_MAGIC_64:
        raise ValueError(f"Not MH_MAGIC_64: 0x{magic:x}")
    if filetype != MH_EXECUTE:
        raise ValueError(f"Not MH_EXECUTE: 0x{filetype:x}")
    if cputype != CPU_TYPE_ARM64:
        raise ValueError(f"Unexpected cputype: 0x{cputype:x}")

    cmds_end = 32 + sizeofcmds
    return ncmds, sizeofcmds, cmds_end


def iter_segments(data: bytes, ncmds: int) -> list[SegmentInfo]:
    off = 32
    segs: list[SegmentInfo] = []
    for i in range(ncmds):
        if off + 8 > len(data):
            raise ValueError("Load command offset out of bounds")
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmdsize == 0:
            raise ValueError(f"Zero cmdsize at command {i}")

        if cmd == LC_SEGMENT_64:
            name_bytes = data[off + 8 : off + 24]
            null_idx = name_bytes.find(b"\x00")
            if null_idx == -1:
                null_idx = len(name_bytes)
            name = name_bytes[:null_idx].decode("ascii", errors="ignore")
            vmaddr, vmsize, fileoff, filesize = struct.unpack_from(
                "<QQQQ", data, off + 24
            )
            segs.append(
                SegmentInfo(
                    name=name,
                    vmaddr=vmaddr,
                    vmsize=vmsize,
                    fileoff=fileoff,
                    filesize=filesize,
                )
            )

        off += cmdsize
    return segs


def header_slack(data: bytes, ncmds: int, sizeofcmds: int) -> int:
    first_sect: int | None = None
    off = 32
    for i in range(ncmds):
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmd == LC_SEGMENT_64:
            nsects = struct.unpack_from("<I", data, off + 64)[0]
            so = off + SEG_CMD_SIZE
            for s in range(nsects):
                sect_off = struct.unpack_from("<I", data, so + 48)[0]
                if sect_off != 0 and (first_sect is None or sect_off < first_sect):
                    first_sect = sect_off
                so += SECT_SIZE
        off += cmdsize

    if first_sect is None:
        raise ValueError("No sections with file data found in Mach-O")

    cmds_end = 32 + sizeofcmds
    if first_sect < cmds_end:
        raise ValueError(
            f"Load commands overlap section data (first sect: {first_sect}, cmdsEnd: {cmds_end})"
        )
    return first_sect - cmds_end


def pack_section(
    sectname: str, segname: str, addr: int, size: int, offset: int
) -> bytes:
    buf = bytearray(SECT_SIZE)
    sname = sectname.encode("ascii")[:16]
    gname = segname.encode("ascii")[:16]
    buf[0 : len(sname)] = sname
    buf[16 : 16 + len(gname)] = gname
    struct.pack_into("<QQI", buf, 32, addr, size, offset)
    return bytes(buf)


def pack_segment(
    segname: str,
    vmaddr: int,
    vmsize: int,
    fileoff: int,
    filesize: int,
    maxprot: int,
    initprot: int,
    sections: list[bytes],
) -> bytes:
    nsects = len(sections)
    cmdsize = SEG_CMD_SIZE + nsects * SECT_SIZE
    buf = bytearray(SEG_CMD_SIZE)
    struct.pack_into("<II", buf, 0, LC_SEGMENT_64, cmdsize)
    gname = segname.encode("ascii")[:16]
    buf[8 : 8 + len(gname)] = gname
    struct.pack_into(
        "<QQQQIII",
        buf,
        24,
        vmaddr,
        vmsize,
        fileoff,
        filesize,
        maxprot,
        initprot,
        nsects,
    )
    result = buf + b"".join(sections)
    return bytes(result)


def build_info_plist(raw_uuid: bytes, corecrypto_kmod: int) -> bytes:
    b64 = base64.b64encode(raw_uuid).decode("ascii")

    prelink_dicts = ""
    if corecrypto_kmod != 0:
        prelink_dicts = f"""
\t\t<dict>
\t\t\t<key>CFBundleIdentifier</key>
\t\t\t<string>com.apple.kec.corecrypto</string>
\t\t\t<key>CFBundlePackageType</key>
\t\t\t<string>KEXT</string>
\t\t\t<key>CFBundleVersion</key>
\t\t\t<string>1.0</string>
\t\t\t<key>AppleKernelExternalComponent</key>
\t\t\t<true/>
\t\t\t<key>OSBundleRequired</key>
\t\t\t<string>Root</string>
\t\t\t<key>_PrelinkKmodInfo</key>
\t\t\t<integer>{corecrypto_kmod}</integer>
\t\t\t<key>_PrelinkExecutableLoadAddr</key>
\t\t\t<integer>0</integer>
\t\t\t<key>_PrelinkExecutableSize</key>
\t\t\t<integer>0</integer>
\t\t</dict>"""

    xml = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
\t<key>_PrelinkInfoDictionary</key>
\t<array>{prelink_dicts}
\t</array>
\t<key>_PrelinkKCID</key>
\t<data>
\t{b64}
\t</data>
</dict>
</plist>
"""
    data = bytearray(xml.encode("utf-8"))
    data.append(0)  # Null terminated
    return bytes(data)


def find_symbol(data: bytes, symbol_name: str) -> tuple[int, bool]:
    if len(data) < 32:
        return 0, False

    ncmds = struct.unpack_from("<I", data, 16)[0]
    off = 32
    for i in range(ncmds):
        if off + 8 > len(data):
            break
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmdsize == 0:
            break
        if cmd == LC_SYMTAB:
            symoff, nsyms, stroff, strsize = struct.unpack_from("<IIII", data, off + 8)
            if stroff + strsize > len(data):
                break
            str_table = data[stroff : stroff + strsize]

            for s in range(nsyms):
                entry_off = symoff + s * 16
                if entry_off + 16 > len(data):
                    break
                strx = struct.unpack_from("<I", data, entry_off)[0]
                val = struct.unpack_from("<Q", data, entry_off + 8)[0]
                if strx < len(str_table):
                    end = str_table.find(b"\x00", strx)
                    if end != -1:
                        name = str_table[strx:end].decode("ascii", errors="ignore")
                        if name == symbol_name and val != 0:
                            return val, True
        off += cmdsize
    return 0, False


def assemble_kernelcache(kernel_data: bytes, raw_uuid: bytes) -> bytes:
    # NOTE: This intentionally does NOT staple a synthetic __PRELINK_INFO
    # segment onto the kernel. libsa/bootstrap.cpp's
    # KLDBootstrap::readStartupExtensions() checks this section's *size* to
    # decide whether to treat the image as a real prelinked kernel
    # (readPrelinkedExtensions(), which unserializes _PrelinkInfoDictionary
    # and expects real per-kext Mach-O data) or a booter/mkext image
    # (readBooterExtensions(), which just reads /chosen/memory-map -- the
    # path this fork's boot flow actually supports). A previous version of
    # this tool added a fake __PRELINK_INFO with a nonzero size purely for
    # descriptive/logging purposes; that made readStartupExtensions() take
    # the real-prelinked-kernel branch on a KC that was never actually
    # kxld-linked, which hung inside readPrelinkedExtensions(). Just pass
    # the kernel through unchanged (helpers above are kept for reference /
    # future real prelinking work).
    del raw_uuid
    return kernel_data


def build_kernelcache(
    kernel_path: str, output_path: str, uuid_str: str | None = None
) -> None:
    with open(kernel_path, "rb") as f:
        kernel_data = f.read()

    if uuid_str:
        hex_str = uuid_str.replace("-", "")
        raw_uuid = bytes.fromhex(hex_str)
    else:
        raw_uuid = secrets.token_bytes(16)

    kc_data = assemble_kernelcache(kernel_data, raw_uuid)

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(kc_data)

    import uuid

    uuid_formatted = str(uuid.UUID(bytes=raw_uuid))
    print(f"kernelcache: {output_path} uuid={uuid_formatted} bytes={len(kc_data)}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Mach-O Kernelcache packaging tool")
    parser.add_argument("kernel", help="Input Mach-O kernel binary")
    parser.add_argument("-o", "--output", required=True, help="Output kernelcache file")
    parser.add_argument("--uuid", help="Explicit UUID for kernelcache")
    args = parser.parse_args()

    build_kernelcache(args.kernel, args.output, args.uuid)


if __name__ == "__main__":
    main()
