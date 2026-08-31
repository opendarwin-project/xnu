project('llvm-project', ['c'], meson_version: '>=1.12.0')

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

prof_dir = meson.current_source_dir() / 'compiler-rt/lib/profile'
inc_dir = meson.current_source_dir() / 'compiler-rt/include'

c_sources = files(
    prof_dir / 'InstrProfiling.c',
    prof_dir / 'InstrProfilingInternal.c',
    prof_dir / 'InstrProfilingPlatformDarwin.c',
    prof_dir / 'InstrProfilingBuffer.c',
    prof_dir / 'InstrProfilingWriter.c',
    prof_dir / 'InstrProfilingNameVar.c',
    prof_dir / 'InstrProfilingVersionVar.c',
)

rt_includes = [
    '-I' + prof_dir,
    '-I' + inc_dir,
    '-I' + srcroot,
    '-I' + buildroot,
    '-I' + gen_dir,
    '-I' + srcroot / 'osfmk',
    '-I' + gen_dir / 'export' / 'osfmk',
    '-I' + srcroot / 'bsd',
    '-I' + srcroot / 'iokit',
    '-I' + srcroot / 'libkern',
    '-I' + srcroot / 'libsa',
    '-I' + srcroot / 'pexpert',
    '-I' + srcroot / 'security',
    '-I' + gen_dir / 'export',
    '-I' + srcroot / 'EXTERNAL_HEADERS',
    '-I' + buildroot / 'syscalls',
    '-I' + buildroot / 'assym',
]

rt_flags = [
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
    '-DKERNEL',
    '-DKERNEL_PRIVATE',
    '-DXNU_KERNEL_PRIVATE',
    '-DPRIVATE',
    '-DCOMPILER_RT_PROFILE_BAREMETAL=1',
    '-DCOMPILER_RT_HAS_ATOMICS=1',
    '-Wno-error',
    '-DXNU_PLATFORM_MacOSX',
    '-DXNU_TARGET_OS_OSX',
    '-include',
    gen_dir / 'master_config.h',
    '-std=gnu17',
]

xnu_compiler_rt = static_library(
    'xnu_compiler_rt',
    c_sources,
    c_args: rt_includes + rt_flags,
    pic: false,
    implicit_include_directories: false,
)
