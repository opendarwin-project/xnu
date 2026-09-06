project(
    'xnu',
    ['c', 'cpp'],
    version: '24.0.0',
    meson_version: '>=1.12.0',
    default_options: [
        'c_std=gnu17',
        'cpp_std=gnu++26',
        'warning_level=1',
        'werror=false',
        'b_staticpic=false',
        'b_pie=false',
        'b_asneeded=false',
        'b_ndebug=false',
        'optimization=s',
    ],
)

python = find_program('python3')
sh = find_program('sh')
cargo = find_program('cargo', required: false)

cc = meson.get_compiler('c')
cxx = meson.get_compiler('cpp')

# Prefer the Xcode toolchain copies for mig/migcom/iig if available
xctoolchain = '/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr'
mig = find_program(xctoolchain / 'bin' / 'mig', 'mig')
migcom = find_program(xctoolchain / 'libexec' / 'migcom', 'migcom')
iig = find_program(xctoolchain / 'bin' / 'iig', 'iig')

kcfg = get_option('kernel_config')
board_opt = get_option('board')
srcroot = meson.project_source_root()
buildroot = meson.project_build_root()

# Target architecture and triple from cross file properties or host machine
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

link_addr = get_option('kernel_link_addr')
plk_addr = get_option('kernel_plkseg_addr')
linkedit_addr = get_option('kernel_linkedit_addr')

# Board specifications
boards = {
    'qemu': {
        'cpu': 'cortex-a53',
        'board_config': 'QEMU',
        'machine_config_lc': 'qemu',
        'icu_source': 'gic/gic.c',
        'extra_defines': ['-DQEMU'],
    },
    'superbird': {
        'cpu': 'cortex-a53',
        'board_config': 'SUPERBIRD',
        'machine_config_lc': 'superbird',
        'icu_source': 'gic/gic.c',
        'extra_defines': ['-DSUPERBIRD'],
    },
    'ipad41': {
        'cpu': 'cyclone',
        'board_config': 'IPAD41',
        'machine_config_lc': 'ipad41',
        'icu_source': 'aic/aic.c',
        'extra_defines': ['-DAPPLECYCLONE'],
    },
}

board = boards[board_opt]

board_defines = [
    '-DCURRENT_MACHINE_CONFIG_LC=' + board['machine_config_lc'],
    '-DARM64_BOARD_CONFIG_' + board['board_config'],
    '-D' + board['board_config'],
] + board['extra_defines']

gen_dir = buildroot / 'gen'
syscalls_dir = buildroot / 'syscalls'
mig_dir = buildroot / 'mig'
mig_ks_dir = mig_dir / 'ks'
iig_dir = buildroot / 'iig'
assym_dir = buildroot / 'assym'

# 1. Generate master config and meta features
xnu_config = files('tools/meson/xnu_config.py')
run_command(
    python,
    xnu_config,
    '--srcroot',
    srcroot,
    '--kernel-config',
    kcfg,
    '--board',
    board_opt,
    '--outdir',
    gen_dir,
    '--format',
    'stats',
    check: true,
)

mig_config_defines = run_command(
    python,
    xnu_config,
    '--srcroot',
    srcroot,
    '--kernel-config',
    kcfg,
    '--board',
    board_opt,
    '--format',
    'mig-defines',
    check: true,
).stdout().strip().split('\n')
if mig_config_defines == ['']
    mig_config_defines = []
endif

# Common compiler flags
common_args = [
    '-target',
    target_triple,
    '-mcpu=' + board['cpu'],
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
    '-DOS_FIREHOSE_SPI=1',
    '-D__MACHO__=1',
    '-DXNU_KERN_EVENT_DATA_IS_VLA',
    '-DXNU_PLATFORM_MacOSX',
    '-DXNU_TARGET_OS_OSX',
    '-D__API__=v4',
    '-fno-aarch64-jump-table-hardening',
    '-Wno-unknown-warning-option',
    '-Wno-unused-command-line-argument',
    '-Wno-nonportable-include-path',
    '-include',
    gen_dir / 'master_config.h',
    '-DVM_KERNEL_LINK_ADDRESS=' + link_addr,
] + board_defines

if get_option('werror')
    common_args += ['-Werror']
endif

common_includes = [
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
    '-I' + syscalls_dir,
    '-I' + assym_dir,
]

c_args = common_args + common_includes + ['-std=gnu17']
cxx_args = common_args + common_includes + [
    '-std=gnu++2c',
    '-fno-rtti',
    '-fno-c++-static-destructors',
    '-fsized-deallocation',
    '-Wno-enum-enum-conversion',
]
s_args = common_args + common_includes + [
    '-x',
    'assembler-with-cpp',
    '-D__ASSEMBLER__',
    '-DASSEMBLER',
]

comp_cflags = {
    'osfmk': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DMACH_KERNEL_PRIVATE',
        '-DMACH_KERNEL',
        '-I' + srcroot / 'osfmk' / 'libsa',
    ],
    'bsd': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DDRIVER_PRIVATE',
        '-D_KERNEL_BUILD',
        '-DKERNEL_BUILD',
        '-DMACH_KERNEL',
        '-DBSD_BUILD',
        '-DBSD_KERNEL_PRIVATE',
    ],
    'iokit': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DDRIVER_PRIVATE',
        '-DIOKIT_KERNEL_PRIVATE',
        '-DDRIVERKIT_PRIVATE=1',
        '-DIOMATCHDEBUG=1',
        '-DIOALLOCDEBUG=1',
        '-DPRIVATE_WIFI_ONLY=1',
        '-DIOASSERT=1',
    ],
    'libkern': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DLIBKERN_KERNEL_PRIVATE',
        '-DOSALLOCDEBUG=1',
    ],
    'libsa': ['-include', gen_dir / 'meta_features.h', '-DLIBSA_KERNEL_PRIVATE'],
    'pexpert': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DPEXPERT_KERNEL_PRIVATE',
    ],
    'security': [
        '-include',
        gen_dir / 'meta_features.h',
        '-DBSD_KERNEL_PRIVATE',
    ],
    'san': ['-include', gen_dir / 'meta_features.h'],
}

# --- BSD Syscalls ---
syscalls_gen = custom_target(
    'bsd-syscalls',
    output: [
        'init_sysent.c',
        'syscalls.c',
        'audit_kevents.c',
        'systrace_args.c',
        'syscall.h',
        'sysproto.h',
    ],
    input: files('bsd/kern/syscalls.master'),
    command: [
        sh,
        files('tools/meson/run_makesyscalls.sh'),
        '@OUTDIR@',
        '@INPUT@',
        files('bsd/kern/makesyscalls.sh'),
        gen_dir,
    ],
)

ioconf_c = custom_target(
    'ioconf.c',
    output: 'ioconf.c',
    command: [
        python,
        '-c',
        'import shutil,sys; shutil.copy(sys.argv[1], sys.argv[2])',
        gen_dir / 'ioconf.c',
        '@OUTPUT@',
    ],
)

version_c = custom_target(
    'version.c',
    output: 'version.c',
    input: files('config/version.c.template'),
    command: [
        python,
        files('tools/meson/stamp_version.py'),
        '--template',
        '@INPUT@',
        '--output',
        '@OUTPUT@',
        '--config',
        kcfg,
        '--version',
        meson.project_version(),
        '--objroot',
        buildroot,
    ],
)

# --- MIG IPC ---
mig_inc = [
    '-I' + srcroot / 'osfmk',
    '-I' + srcroot / 'bsd',
    '-I' + srcroot / 'EXTERNAL_HEADERS',
    '-I' + gen_dir,
]
mig_base = [
    mig,
    '-novouchers',
    '-migcom',
    migcom,
    '-DAPPLE',
    '-DKERNEL',
    '-DKERNEL_PRIVATE',
    '-DXNU_KERNEL_PRIVATE',
    '-DPRIVATE',
    '-DMACH_KERNEL_PRIVATE',
    '-DXNU_TARGET_OS_OSX',
    '-target',
    target_triple,
] + mig_inc + mig_config_defines

mig_jobs_def = [
    ['mach/clock.defs', 'server', 'clock_server.c'],
    ['mach/clock_reply.defs', 'user', 'clock_reply_user.c'],
    ['mach/exc.defs', 'user', 'exc_user.c'],
    ['mach/exc.defs', 'server', 'exc_server.c'],
    ['mach/mach_eventlink.defs', 'server', 'mach_eventlink_server.c'],
    ['mach/host_priv.defs', 'server', 'host_priv_server.c'],
    ['mach/host_notify_reply.defs', 'user', 'host_notify_reply_user.c'],
    ['mach/ktrace_background.defs', 'user', 'ktrace_background_user.c'],
    ['mach/mach_exc.defs', 'user', 'mach_exc_user.c'],
    ['mach/mach_exc.defs', 'server', 'mach_exc_server.c'],
    ['mach/mach_host.defs', 'server', 'mach_host_server.c'],
    ['mach/notify.defs', 'user', 'mach_notify_user.c'],
    ['mach/mach_port.defs', 'server', 'mach_port_server.c'],
    ['mach/mach_vm.defs', 'server', 'mach_vm_server.c'],
    ['mach/mach_voucher.defs', 'server', 'mach_voucher_server.c'],
    ['mach/memory_entry.defs', 'server', 'memory_entry_server.c'],
    ['mach/resource_notify.defs', 'user', 'resource_notify_user.c'],
    ['mach/restartable.defs', 'server', 'restartable_server.c'],
    ['mach/audit_triggers.defs', 'user', 'audit_triggers_user.c'],
    ['mach/task_access.defs', 'user', 'task_access_user.c'],
    [
        'mach/telemetry_notification.defs',
        'user',
        'telemetry_notification_user.c',
    ],
    [
        'mach/coalition_notification.defs',
        'user',
        'coalition_notification_user.c',
    ],
    [
        'mach/sysdiagnose_notification.defs',
        'user',
        'sysdiagnose_notification_user.c',
    ],
    ['mach/vfs_nspace.defs', 'user', 'vfs_nspace_user.c'],
    [
        'mach/fairplayd_notification.defs',
        'user',
        'fairplayd_notification_user.c',
    ],
    ['mach/arcade_upcall.defs', 'user', 'arcade_upcall_user.c'],
    ['mach/arcade_register.defs', 'server', 'arcade_register_server.c'],
    [
        'mach/memory_error_notification.defs',
        'user',
        'memory_error_notification_user.c',
    ],
    ['mach/doubleagent_mig.defs', 'user', 'doubleagent_mig_user.c'],
    [
        'mach/iocompressionstats_notification.defs',
        'user',
        'iocompressionstats_notification_user.c',
    ],
    ['mach/processor.defs', 'server', 'processor_server.c'],
    ['mach/processor_set.defs', 'server', 'processor_set_server.c'],
    ['mach/task.defs', 'server', 'task_server.c'],
    ['mach/thread_act.defs', 'server', 'thread_act_server.c'],
    ['mach/vm32_map.defs', 'server', 'vm32_map_server.c'],
    ['mach/vm_map.defs', 'server', 'vm_map_server.c'],
    ['mach/upl.defs', 'server', 'upl_server.c'],
    ['mach/mach_test_upcall.defs', 'user', 'mach_test_upcall_user.c'],
    ['device/device.defs', 'server', 'device_server.c'],
    ['UserNotification/UNDRequest.defs', 'user_plain', 'UNDRequest.c'],
    ['UserNotification/UNDReply.defs', 'server_plain', 'UNDReplyServer.c'],
    ['gssd/gssd_mach.defs', 'user_plain', 'gssd_mach.c'],
    ['lockd/lockd_mach.defs', 'user_plain', 'lockd_mach.c'],
    ['kextd/kextd_mach.defs', 'user_plain', 'kextd_mach.c'],
    ['mach/clock.defs', 'uuhdr', 'clock.h'],
    ['mach/clock_priv.defs', 'uuhdr', 'clock_priv.h'],
    ['mach/host_priv.defs', 'uuhdr', 'host_priv.h'],
    ['mach/host_security.defs', 'uuhdr', 'host_security.h'],
    ['mach/mach_host.defs', 'uuhdr', 'mach_host.h'],
    ['mach/mach_port.defs', 'uuhdr', 'mach_port.h'],
    ['mach/mach_vm.defs', 'uuhdr', 'mach_vm.h'],
    ['mach/mach_voucher.defs', 'uuhdr', 'mach_voucher.h'],
    [
        'mach/mach_voucher_attr_control.defs',
        'uuhdr',
        'mach_voucher_attr_control.h',
    ],
    ['mach/memory_entry.defs', 'uuhdr', 'memory_entry.h'],
    ['mach/processor.defs', 'uuhdr', 'processor.h'],
    ['mach/processor_set.defs', 'uuhdr', 'processor_set.h'],
    ['mach/task.defs', 'uuhdr', 'task.h'],
    ['mach/thread_act.defs', 'uuhdr', 'thread_act.h'],
    ['mach/upl.defs', 'uuhdr', 'upl.h'],
    ['mach/vm_map.defs', 'uuhdr', 'vm_map.h'],
    ['mach/audit_triggers.defs', 'ushdr', 'audit_triggers_server.h'],
    ['mach/clock_reply.defs', 'ushdr', 'clock_reply_server.h'],
    [
        'mach/coalition_notification.defs',
        'ushdr',
        'coalition_notification_server.h',
    ],
    ['mach/doubleagent_mig.defs', 'ushdr', 'doubleagent_mig_server.h'],
    ['mach/exc.defs', 'ushdr', 'exc_server.h'],
    [
        'mach/fairplayd_notification.defs',
        'ushdr',
        'fairplayd_notification_server.h',
    ],
    ['mach/arcade_upcall.defs', 'ushdr', 'arcade_upcall_server.h'],
    [
        'mach/memory_error_notification.defs',
        'ushdr',
        'memory_error_notification_server.h',
    ],
    ['mach/mach_exc.defs', 'ushdr', 'mach_exc_server.h'],
    ['mach/notify.defs', 'ushdr', 'notify_server.h'],
    ['mach/task_access.defs', 'ushdr', 'task_access_server.h'],
    [
        'mach/telemetry_notification.defs',
        'ushdr',
        'telemetry_notification_server.h',
    ],
    [
        'mach/sysdiagnose_notification.defs',
        'ushdr',
        'sysdiagnose_notification_server.h',
    ],
    [
        'mach/iocompressionstats_notification.defs',
        'ushdr',
        'iocompressionstats_notification_server.h',
    ],
    ['mach/vfs_nspace.defs', 'ushdr', 'vfs_nspace_server.h'],
]

mig_c = []
mig_hdrs = []
mig_subdirs = ['mach', 'device', 'UserNotification', 'gssd', 'lockd', 'kextd']

foreach job : mig_jobs_def
    defs = job[0]
    kind = job[1]
    outc = job[2]
    stem = outc.replace('.c', '')
    sub = defs.split('/')[0]
    out_dir = mig_dir / sub
    ks = ['-DMACH_KERNEL_PRIVATE', '-DKERNEL_SERVER=1', '-mach_msg2']
    ku = [
        '-DMACH_KERNEL_PRIVATE',
        '-DKERNEL_USER=1',
        '-DEXC_SERVER_AUDITTOKEN=1',
        '-DMACH_EXC_SERVER_AUDITTOKEN=1',
        '-maxonstack',
        '1024',
    ]
    if kind == 'server'
        extra = ks + [
            '-user',
            '/dev/null',
            '-header',
            '/dev/null',
            '-server',
            '@OUTPUT@',
            '-sheader',
            mig_ks_dir / sub / (stem + '.h'),
            srcroot / 'osfmk' / defs,
        ]
    elif kind == 'user'
        extra = ku + [
            '-user',
            '@OUTPUT@',
            '-header',
            out_dir / (outc.replace('_user.c', '.h')),
            '-server',
            '/dev/null',
            '-sheader',
            '/dev/null',
            srcroot / 'osfmk' / defs,
        ]
    elif kind == 'user_plain'
        extra = ku + [
            '-user',
            '@OUTPUT@',
            '-header',
            out_dir / (stem + '.h'),
            '-server',
            '/dev/null',
            '-sheader',
            '/dev/null',
            srcroot / 'osfmk' / defs,
        ]
    elif kind == 'server_plain'
        extra = ks + [
            '-user',
            '/dev/null',
            '-header',
            '/dev/null',
            '-server',
            '@OUTPUT@',
            '-sheader',
            out_dir / (stem + '.h'),
            srcroot / 'osfmk' / defs,
        ]
    elif kind == 'uuhdr'
        extra = [
            '-user',
            '/dev/null',
            '-header',
            out_dir / outc,
            '-server',
            '/dev/null',
            '-sheader',
            '/dev/null',
            srcroot / 'osfmk' / defs,
        ]
    elif kind == 'ushdr'
        extra = [
            '-user',
            '/dev/null',
            '-header',
            '/dev/null',
            '-server',
            '/dev/null',
            '-sheader',
            out_dir / outc,
            srcroot / 'osfmk' / defs,
        ]
    else
        extra = ks + [
            '-user',
            '/dev/null',
            '-header',
            '/dev/null',
            '-server',
            '@OUTPUT@',
            '-sheader',
            mig_ks_dir / sub / (stem + '.h'),
            srcroot / 'osfmk' / defs,
        ]
    endif

    mig_t = custom_target(
        'mig-' + outc,
        output: outc,
        input: files('osfmk/' + defs),
        command: [
            python,
            '-c',
            'import os,sys,subprocess; os.makedirs(sys.argv[1], exist_ok=True); os.makedirs(sys.argv[2], exist_ok=True); subprocess.check_call(sys.argv[3:])',
            out_dir,
            mig_ks_dir / sub,
        ] + mig_base + extra,
        depend_files: files(
            'osfmk/mach/std_types.defs',
            'osfmk/mach/mach_types.defs',
        ),
    )
    if kind not in ['uuhdr', 'ushdr']
        mig_c += mig_t
    else
        mig_hdrs += mig_t
    endif
endforeach

mig_inc_dirs = [
    '-I' + mig_dir,
]
foreach sub : mig_subdirs
    mig_inc_dirs += ['-I' + mig_dir / sub]
endforeach

mig_ks_incs = [
    '-I' + mig_ks_dir,
]
foreach sub : mig_subdirs
    mig_ks_incs += ['-I' + mig_ks_dir / sub]
endforeach

# --- IIG (DriverKit) ---
iig_c = []
iig_hdrs = []
iig_defs = [
    'OSObject',
    'OSAction',
    'IOService',
    'IOExtensiblePaniclog',
    'IOUserClient',
    'IOMemoryDescriptor',
    'IOBufferMemoryDescriptor',
    'IOMemoryMap',
    'IODispatchSource',
    'IODispatchQueue',
    'IOInterruptDispatchSource',
    'IODataQueueDispatchSource',
    'IOServiceNotificationDispatchSource',
    'IOServiceStateNotificationDispatchSource',
    'IOEventLink',
    'IOWorkGroup',
    'IOUserServer',
    'IODMACommand',
]
iig_hdr_dir = gen_dir / 'export' / 'DriverKit'
iig_cflags = [
    '-x',
    'c++',
    '-std=gnu++2c',
    '-D__IIG=1',
    '-DDRIVERKIT_PRIVATE=1',
    '-DPRIVATE_WIFI_ONLY=1',
    '-I' + srcroot / 'iokit',
    '-I' + srcroot / 'osfmk',
    '-I' + srcroot / 'bsd',
    '-I' + srcroot / 'EXTERNAL_HEADERS',
    '-I' + gen_dir,
    '-I' + gen_dir / 'export',
    '-I' + syscalls_dir,
]
foreach name : iig_defs
    iig_t = custom_target(
        'iig-' + name,
        output: [name + '.iig.cpp', name + '.h'],
        input: files('iokit/DriverKit/' + name + '.iig'),
        command: [
            python,
            '-c',
            'import os,sys,subprocess,shutil; os.makedirs(sys.argv[1], exist_ok=True); subprocess.check_call(sys.argv[4:]); shutil.copy2(sys.argv[3], os.path.join(sys.argv[1], os.path.basename(sys.argv[3])))',
            iig_hdr_dir,
            '@OUTPUT0@',
            '@OUTPUT1@',
            iig,
            '--def',
            '@INPUT@',
            '--header',
            '@OUTPUT1@',
            '--impl',
            '@OUTPUT0@',
            '--framework-name',
            'DriverKit',
            '--',
        ] + iig_cflags,
    )
    iig_c += iig_t[0]
    iig_hdrs += iig_t[1]
endforeach

iig_inc_dirs = [
    '-I' + iig_dir,
    '-I' + gen_dir / 'export',
]

# Update c_args and cxx_args with MIG and IIG include directories
c_args += mig_inc_dirs + iig_inc_dirs
cxx_args += mig_inc_dirs + iig_inc_dirs
s_args += mig_inc_dirs + iig_inc_dirs

# Genassym
genassym_s = custom_target(
    'genassym.s',
    output: 'genassym.s',
    input: files('osfmk/arm64/genassym.c'),
    command: cc.cmd_array() + ['-S', '-fno-integrated-as', '-o', '@OUTPUT@'] + c_args + comp_cflags['osfmk'] + [
        '@INPUT@',
    ],
    depends: mig_hdrs,
)
assym_h = custom_target(
    'assym.h',
    output: 'assym.h',
    input: genassym_s,
    command: [
        python,
        files('tools/meson/extract_assym.py'),
        '@INPUT@',
        '@OUTPUT@',
        gen_dir / 'assym.s',
    ],
)

gen_order = mig_hdrs + iig_hdrs + [assym_h]

lib_kwargs = {
    'pic': false,
    'implicit_include_directories': false,
}

kernel_libs = []

# --- Subsystems ---
subdir('osfmk')
subdir('bsd')
# libkern must link before iokit!
subdir('libkern')
subdir('iokit')
subdir('libsa')
subdir('pexpert')
subdir('security')
subdir('san')

# --- Subprojects ---
corecrypto_sp = subproject('corecrypto')
kernel_libs += corecrypto_sp.get_variable('xnu_corecrypto')

iostorage_sp = subproject('iostorage')
kernel_libs += iostorage_sp.get_variable('xnu_iostorage')

msdosfs_sp = subproject('msdosfs')
kernel_libs += msdosfs_sp.get_variable('xnu_msdosfs')

libpthread_sp = subproject('libpthread')
kernel_libs += libpthread_sp.get_variable('xnu_libpthread')

compiler_rt_sp = subproject('llvm-project')
kernel_libs += compiler_rt_sp.get_variable('xnu_compiler_rt')

libdispatch_sp = subproject('libdispatch')
kernel_libs += libdispatch_sp.get_variable('xnu_libdispatch')

# lastkernelconstructor must be the last object in __mod_init_func so
# iokit_post_constructor_init() runs after every other C++ constructor.
# Nix sorts libsa last for the same reason; linking it from libxnu_libsa.a
# in the middle of link_whole left 12 constructors after it and hung in
# InitIOKit (garbled serial after "iBoot Stage 2 version").
xnu_lastkernelconstructor = static_library(
    'xnu_lastkernelconstructor',
    files('libsa/lastkernelconstructor.c'),
    c_args: c_args + comp_cflags['libsa'],
    kwargs: lib_kwargs,
)
kernel_libs += [xnu_lastkernelconstructor]

# --- Tools and Userspace ---
subdir('userspace')

# --- Final Kernel Link ---
ldflags = [
    '-target',
    target_triple,
    '-mcpu=' + board['cpu'],
    '-nostdlib',
    '-nostdlib++',
    '-static',
    '-Wl,-e,__start',
    '-Wl,-pie',
    '-Wl,-image_base,' + link_addr,
    '-Wl,-segaddr,__PRELINK_TEXT,' + plk_addr,
    '-Wl,-segalign,0x1000',
    '-Wl,-pagezero_size,0x0',
    '-Wl,-rename_section,__TEXT,__text,__TEXT_EXEC,__text',
    '-Wl,-rename_section,__DATA,__const,__DATA_CONST,__const',
    '-Wl,-rename_section,__DATA,__mod_init_func,__DATA_CONST,__mod_init_func',
    '-Wl,-rename_section,__DATA,__mod_term_func,__DATA_CONST,__mod_term_func',
    '-Wl,-segprot,__TEXT,r--,r--',
    '-Wl,-segprot,__TEXT_EXEC,r-x,r-x',
    '-Wl,-segprot,__DATA_CONST,r--,r--',
    '-Wl,-segment_order,__TEXT:__DATA_CONST:__DATA_SPTM:__TEXT_EXEC:__TEXT_BOOT_EXEC:__KLD:__PPLTEXT:__PPLTRAMP:__PPLDATA_CONST:__LASTDATA_CONST:__LAST:__PPLDATA:__KLDDATA:__DATA:__HIBDATA:__BOOTDATA:__LINKEDIT',
    '-Wl,-headerpad,0x1000',
    '-Wl,-segaddr,__LINKEDIT,' + linkedit_addr,
    '-Wl,-no_fixup_chains,',
    '-Wl,-segprot,__LAST,r-x,r-x',
    '-Wl,-segprot,__KLDDATA,rw-,rw-',
    '-Wl,-u,_g_corecrypto_kmod_info',
    '-Wl,-u,_gIOPL031RTCBuiltinInfo',
    '-Wl,-u,__ZN10IOPL031RTC10gMetaClassE',
    '-Wl,-u,__ZN11IOVirtioBlk10gMetaClassE',
    '-Wl,-u,__ZN9IOStorage10gMetaClassE',
    '-Wl,-u,__ZN20IOBlockStorageDriver10gMetaClassE',
    '-Wl,-u,__ZN7IOMedia10gMetaClassE',
    '-Wl,-u,__ZN16IOMediaBSDClient10gMetaClassE',
    '-Wl,-u,__ZN17IOPartitionScheme10gMetaClassE',
    '-Wl,-u,__ZN22IOFDiskPartitionScheme10gMetaClassE',
    '-Wl,-u,__ZN21IOGUIDPartitionScheme10gMetaClassE',
    '-Wl,-u,__ZN24IOUserBlockStorageDevice10gMetaClassE',
]

mach_kernel = executable(
    'mach_kernel',
    files('libsa/lastkerneldataconst.c'),
    c_args: c_args + comp_cflags['libsa'],
    link_language: 'c',
    link_args: ldflags,
    link_whole: kernel_libs,
    pie: false,
    install: false,
)

kernelcache = custom_target(
    'kernelcache',
    input: mach_kernel,
    output: 'kernelcache',
    command: [
        python,
        files('tools/meson/kernelcache.py'),
        '@INPUT@',
        '-o',
        '@OUTPUT@',
    ],
    build_by_default: true,
)

