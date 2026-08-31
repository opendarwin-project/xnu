# QEMU boot loader (Rust aarch64 bare-metal target)
if cargo.found()
    qemu_boot_target = custom_target(
        'qemu-boot',
        output: 'qemu-boot.elf',
        command: [
            python,
            '-c',
            '''import subprocess, shutil, sys
cargo_cmd = [sys.argv[1], "build", "--manifest-path", sys.argv[2], "--target", "aarch64-unknown-none", "--release"]
subprocess.check_call(cargo_cmd)
shutil.copy(sys.argv[3], sys.argv[4])
''',
            cargo,
            meson.project_source_root() / 'tools/qemu-boot/Cargo.toml',
            meson.project_source_root() / 'tools/qemu-boot/target/aarch64-unknown-none/release/qemu-boot',
            '@OUTPUT@',
        ],
        build_by_default: true,
        console: true,
    )
endif
