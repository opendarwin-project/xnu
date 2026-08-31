#!/usr/bin/env python3
"""QEMU runner and bootloader helper for XNU under Meson."""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def find_build_dir(hint: str | None = None) -> Path:
    """Locate the Meson build directory."""
    if hint:
        p = Path(hint)
        if p.exists():
            return p.resolve()
        raise FileNotFoundError(f"Specified build directory '{hint}' does not exist.")

    # Check environment variable
    env_dir = os.environ.get("MESON_BUILD_ROOT") or os.environ.get("BUILD_DIR")
    if env_dir and Path(env_dir).exists():
        return Path(env_dir).resolve()

    # Search common build directory names
    src_root = Path(__file__).resolve().parent.parent
    candidates = [
        src_root / "build_cross",
        src_root / "build_meson",
        src_root / "build",
        src_root / "build_qemu",
        src_root / "build_superbird",
        Path.cwd(),
    ]
    for c in candidates:
        if (c / "build.ninja").exists() or (c / "kernelcache").exists():
            return c.resolve()

    # Default to build_cross or build_meson relative to srcroot
    for c in [src_root / "build_cross", src_root / "build_meson", src_root / "build"]:
        if c.exists():
            return c.resolve()

    return (src_root / "build_cross").resolve()


def find_qemu_bin(override: str | None = None) -> str:
    """Find the qemu-system-aarch64 executable."""
    if override:
        if shutil.which(override) or os.path.exists(override):
            return override
        raise FileNotFoundError(f"Specified QEMU binary '{override}' not found.")

    env_qemu = os.environ.get("QEMU")
    if env_qemu and (shutil.which(env_qemu) or os.path.exists(env_qemu)):
        return env_qemu

    candidates = [
        "/Users/theo/.nix-profile/bin/qemu-system-aarch64",
        "/opt/homebrew/bin/qemu-system-aarch64",
        "/usr/local/bin/qemu-system-aarch64",
        "/usr/bin/qemu-system-aarch64",
    ]
    for cand in candidates:
        if os.path.exists(cand) and os.access(cand, os.X_OK):
            return cand

    which_qemu = shutil.which("qemu-system-aarch64")
    if which_qemu:
        return which_qemu

    raise FileNotFoundError(
        "qemu-system-aarch64 not found in PATH or standard locations."
    )


def build_qemu_boot(src_root: Path, out_dir: Path) -> Path:
    """Build the Rust QEMU bootloader using cargo."""
    boot_elf = out_dir / "tools" / "qemu-boot.elf"
    if not boot_elf.parent.exists():
        boot_elf = out_dir / "qemu-boot.elf"

    manifest_path = src_root / "tools" / "qemu-boot" / "Cargo.toml"
    if not manifest_path.exists():
        raise FileNotFoundError(f"Cargo manifest not found: {manifest_path}")

    print(f"Building Rust QEMU bootloader ({boot_elf})...")
    cmd = [
        "cargo",
        "build",
        "--manifest-path",
        str(manifest_path),
        "--target",
        "aarch64-unknown-none",
        "--release",
    ]
    subprocess.check_call(cmd)

    cargo_elf = (
        src_root
        / "tools"
        / "qemu-boot"
        / "target"
        / "aarch64-unknown-none"
        / "release"
        / "qemu-boot"
    )
    boot_elf.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(cargo_elf, boot_elf)
    return boot_elf


def run_qemu(
    build_dir: str | Path | None = None,
    kernel_path: str | Path | None = None,
    boot_elf: str | Path | None = None,
    init_path: str | Path | None = None,
    rootfs_path: str | Path | None = None,
    cpu: str = "cortex-a53",
    memory: int = 1024,
    smp: int = 2,
    gdb: bool = False,
    gdb_wait: bool = False,
    qemu_bin: str | None = None,
    extra_args: list[str] | None = None,
) -> int:
    """Launch QEMU with the XNU kernel and bootloader."""
    src_root = Path(__file__).resolve().parent.parent
    bdir = find_build_dir(str(build_dir) if build_dir else None)

    qemu = find_qemu_bin(qemu_bin)

    # Locate bootloader
    if boot_elf:
        boot_path = Path(boot_elf).resolve()
    else:
        candidates = [
            bdir / "tools" / "qemu-boot.elf",
            bdir / "qemu-boot.elf",
            src_root
            / "tools"
            / "qemu-boot"
            / "target"
            / "aarch64-unknown-none"
            / "release"
            / "qemu-boot",
        ]
        boot_path = None
        for cand in candidates:
            if cand.exists():
                boot_path = cand
                break
        if not boot_path:
            boot_path = build_qemu_boot(src_root, bdir)

    # Locate kernelcache / mach_kernel
    if kernel_path:
        kpath = Path(kernel_path).resolve()
    else:
        kcand = [
            bdir / "kernelcache",
            bdir / "mach_kernel",
        ]
        kpath = None
        for cand in kcand:
            if cand.exists():
                kpath = cand
                break
        if not kpath:
            raise FileNotFoundError(
                f"Kernel artifact not found in {bdir}. Please build the project first."
            )

    # Locate userspace init
    if init_path:
        ipath = Path(init_path).resolve()
    else:
        icand = [
            bdir / "userspace" / "init",
            bdir / "init",
        ]
        ipath = None
        for cand in icand:
            if cand.exists():
                ipath = cand
                break

    # Locate rootfs
    if rootfs_path:
        rpath = Path(rootfs_path).resolve()
    else:
        rcand = [
            bdir / "rootfs.fat32",
            src_root / "rootfs.fat32",
        ]
        rpath = None
        for cand in rcand:
            if cand.exists():
                rpath = cand
                break

    qemu_args = [
        qemu,
        "-M",
        "virt",
        "-cpu",
        cpu,
        "-m",
        str(memory),
        "-smp",
        str(smp),
        "-nographic",
        "-kernel",
        str(boot_path),
        "-device",
        f"loader,force-raw=on,addr=0x48000000,file={kpath}",
    ]

    if ipath and ipath.exists():
        qemu_args.extend([
            "-device",
            f"loader,force-raw=on,addr=0x50000000,file={ipath}",
        ])

    if rpath and rpath.exists():
        qemu_args.extend([
            "-drive",
            f"file={rpath},if=none,format=raw,id=hd0",
            "-device",
            "virtio-blk-device,drive=hd0",
        ])

    if gdb or gdb_wait:
        qemu_args.append("-s")
    if gdb_wait:
        qemu_args.append("-S")

    if extra_args:
        qemu_args.extend(extra_args)

    print("==> Running QEMU:")
    print("    " + " ".join(str(a) for a in qemu_args))
    return subprocess.run(qemu_args).returncode


def main() -> None:
    parser = argparse.ArgumentParser(description="Run XNU under QEMU (Meson)")
    parser.add_argument(
        "-B",
        "--build-dir",
        help="Meson build directory (default: auto-detect)",
        default=None,
    )
    parser.add_argument(
        "-k",
        "--kernel",
        help="Path to kernelcache / mach_kernel (default: <build-dir>/kernelcache)",
        default=None,
    )
    parser.add_argument(
        "-b",
        "--bootloader",
        help="Path to qemu-boot.elf (default: <build-dir>/tools/qemu-boot.elf)",
        default=None,
    )
    parser.add_argument(
        "-i",
        "--init",
        help="Path to userspace init binary (default: <build-dir>/userspace/init)",
        default=None,
    )
    parser.add_argument(
        "-r",
        "--rootfs",
        help="Path to rootfs.fat32 disk image",
        default=None,
    )
    parser.add_argument(
        "--cpu",
        default="cortex-a53",
        help="Target CPU model (default: cortex-a53)",
    )
    parser.add_argument(
        "-m",
        "--memory",
        type=int,
        default=1024,
        help="Guest RAM in MB (default: 1024)",
    )
    parser.add_argument(
        "-c",
        "--smp",
        type=int,
        default=2,
        help="Number of guest CPU cores (default: 2)",
    )
    parser.add_argument(
        "-s",
        "--gdb",
        action="store_true",
        help="Enable GDB server on TCP port 1234",
    )
    parser.add_argument(
        "-S",
        "--gdb-wait",
        action="store_true",
        help="Freeze CPU at startup and enable GDB server (-s -S)",
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="Build targets using ninja before running QEMU",
    )
    parser.add_argument(
        "--qemu-bin",
        default=None,
        help="Path to qemu-system-aarch64 executable",
    )

    args, unknown = parser.parse_known_args()

    bdir = find_build_dir(args.build_dir)
    if args.build:
        print(f"==> Building XNU in {bdir}...")
        subprocess.check_call(["ninja", "-C", str(bdir)])

    code = run_qemu(
        build_dir=bdir,
        kernel_path=args.kernel,
        boot_elf=args.bootloader,
        init_path=args.init,
        rootfs_path=args.rootfs,
        cpu=args.cpu,
        memory=args.memory,
        smp=args.smp,
        gdb=args.gdb,
        gdb_wait=args.gdb_wait,
        qemu_bin=args.qemu_bin,
        extra_args=unknown,
    )
    sys.exit(code)


if __name__ == "__main__":
    main()
