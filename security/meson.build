security_c_sources = files(
    'mac_audit.c',
    'mac_base.c',
    'mac_data.c',
    'mac_label.c',
    'mac_necp.c',
    'mac_process.c',
    'mac_vfs.c',
    'mac_vfs_subr.c',
    'mac_skywalk.c',
    'mac_system.c',
    'mac_sysv_sem.c',
    'mac_sysv_shm.c',
    'mac_sysv_msg.c',
    'mac_posix_sem.c',
    'mac_posix_shm.c',
    'mac_socket.c',
    'mac_pipe.c',
    'mac_iokit.c',
    'mac_file.c',
    'mac_priv.c',
    'mac_pty.c',
    'mac_kext.c',
    'mac_mach.c',
)

security_cflags = comp_cflags['security'] + c_args

xnu_security = static_library(
    'xnu_security',
    security_c_sources + gen_order,
    c_args: security_cflags,
    kwargs: lib_kwargs,
)

kernel_libs += [xnu_security]
