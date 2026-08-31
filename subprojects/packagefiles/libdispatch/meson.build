project('libdispatch', ['c'], meson_version: '>=1.12.0')

srcroot = meson.global_source_root()
buildroot = meson.global_build_root()
gen_dir = buildroot / 'gen'

target_triple = meson.get_external_property('target_triple', '')
if target_triple == ''
    target_cpu_family = host_machine.cpu_family()
    if target_cpu_family == 'aarch64'
        target_arch = 'arm64'
    elif target_cpu_family == 'x86_64'
        target_arch = 'x86_64'
    else
        target_arch = target_cpu_family
    endif
    target_triple = target_arch + '-apple-macos'
else
    target_arch = target_triple.split('-')[0]
endif

dispatch_sources = files([
    'src/firehose/firehose_buffer.c',
])

dispatch_includes = [
    '-I' + meson.current_source_dir() / 'src' / 'firehose',
    '-I' + srcroot,
    '-I' + buildroot,
    '-I' + gen_dir,
    '-I' + srcroot / 'osfmk',
    '-I' + gen_dir / 'export' / 'osfmk',
    '-I' + srcroot / 'bsd',
    '-I' + srcroot / 'iokit',
    '-I' + srcroot / 'libkern',
    '-I' + srcroot / 'libkern' / 'firehose',
    '-I' + srcroot / 'libsa',
    '-I' + srcroot / 'pexpert',
    '-I' + srcroot / 'security',
    '-I' + gen_dir / 'export',
    '-I' + srcroot / 'EXTERNAL_HEADERS',
    '-I' + buildroot / 'syscalls',
    '-I' + buildroot / 'assym',
]

dispatch_cflags = [
    '-target',
    target_triple,
    '-ffreestanding',
    '-fno-builtin',
    '-fno-common',
    '-fno-stack-protector',
    '-fno-strict-aliasing',
    '-fno-exceptions',
    '-momit-leaf-frame-pointer',
    '-nostdlibinc',
    '-ftrivial-auto-var-init=zero',
    '-fmerge-all-constants',
    '-Os',
    '-D' + target_arch,
    '-D' + target_arch.to_upper(),
    '-D__' + target_arch.to_upper() + '__',
    '-DLP64',
    '-DPAGE_SIZE_FIXED',
    '-UARM_LARGE_MEMORY',
    '-UARM_LARGE_MEMORY_KERNONLY',
    '-DAPPLE',
    '-DKERNEL=1',
    '-DKERNEL_PRIVATE=1',
    '-DXNU_KERNEL_PRIVATE=1',
    '-DPRIVATE=1',
    '-DOS_FIREHOSE_SPI=1',
    '-D__MACHO__=1',
    '-Wno-visibility',
    '-include',
    gen_dir / 'master_config.h',
    '-include',
    gen_dir / 'meta_features.h',
    '-std=gnu17',
]

xnu_libdispatch = static_library(
    'xnu_libdispatch',
    dispatch_sources,
    c_args: dispatch_includes + dispatch_cflags,
    pic: false,
    implicit_include_directories: false,
)
