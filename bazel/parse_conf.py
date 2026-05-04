#!/usr/bin/env python3
"""Parse XNU BSD-style conf/files into a list of sources matching enabled features."""
import sys, re, os
from pathlib import Path

# Enabled features (boolean true). Features in DUMMY_HEADERS are disabled.
# Defaults from BUILD.bazel COMMON_KERNEL_FLAGS + X86_64_FLAGS.
ENABLED = {
    # Compile-time enabled defines
    "mach_bsd",
    "mach_kdp",
    "kperf",
    "hibernation",
    "crypto",
    # Filesystem / IO basics needed for kernel link
    "iokit",
    "iokitcpp",
    "libkerncpp",
    "config_blocks",
    "ptmx", "pty",
    "fdesc", "fifo", "devfs", "routefs",
    "config_macf",
    "sockets", "networking", "inet", "ether", "loop",
    "bpfilter",
    "config_xnupost", "development", "debug",  # may actually be off
    "fb",
    "config_clutch",
    "ipv6send",
    "vlan", "bond",
    "if_fake", "if_headless", "if_redirect", "if_bridge", "bridgestp", "gif",
    "necp", "ipsec", "stf",
    "multipath", "mptcp",
    "dummynet",
    "sendfile",
    "pf", "pflog",
    "config_telemetry",
    "config_imageboot",
    "config_imageboot_chunklist",
    "config_csr",
    "config_audit",
    "config_io_compression_stats",
    "config_personas",
    "config_proc_uuid_policy",
    "config_coredump", "config_ucoredump",
    "config_coalitions",
    "config_phantom_cache",
    "config_arcade",
    "config_atm",
    "config_deferred_reclaim",
    "config_kxld",
    "config_mtrr",
    "config_vmx",
    "config_mca",
    "config_mbuf_mcache",
    "config_memorystatus",
    "config_requires_u32_munging",
    "skywalk",
    "config_nexus_user_pipe",
    "config_nexus_kernel_pipe",
    "config_nexus_flowswitch",
    "config_nexus_netif",
    "content_filter",
    "packet_mangler",
    "kctl_test",
    "remote_vif",
    "fs_compression",
    "quota",
    "kdebug",
    "sysv_sem", "sysv_msg", "sysv_shm",
    "config_mach_bridge_send_time",
    "hypervisor",
    "config_sysdiagnose",
    "config_pv_ticket",
    "zlib",
    "pgo",
    "config_iotrace",
    "config_exclaves",
    "config_netboot",
    "nfsserver",
    "nullfs", "bindfs", "mockfs",
    "config_triggers",
    "diagnostic", "profiling",
    "config_ecc_logging",
    "pal_i386",
    "config_audit",
    "mig_debug",
    "iotracking",
    # Disabled  - explicitly NOT in this set:
    # mach_assert, mach_ldebug, debug, zleaks, config_serial_kdp,
    # importance_inheritance, importance_debug, config_dtrace, no_kextd,
    # config_quiesce_counter, config_cpu_counters, config_user_notification,
    # config_kdp_interactive_debugging, config_voucher_deprecated,
    # config_kdp_coredump_encryption
}

DISABLED = {
    "mach_assert", "mach_ldebug", "zleaks", "config_serial_kdp",
    "importance_inheritance", "importance_debug", "config_dtrace", "no_kextd",
    "config_quiesce_counter", "config_cpu_counters", "config_user_notification",
    "config_kdp_interactive_debugging", "config_voucher_deprecated",
    "config_kdp_coredump_encryption",
    "copyout_shim", "kctl_test",
    "libkerncpp",  # too much C++ scaffolding pulled in
    "iokitcpp",    # ditto
    "config_blocks",
    "iotracking",
    "skywalk", "config_nexus_user_pipe", "config_nexus_kernel_pipe",
    "config_nexus_flowswitch", "config_nexus_netif",
    "config_xnupost", "development",
    "config_kxld",
    "pgo",
    "config_arcade",
    "config_atm",
    "config_audit",
    "config_macf",
    "config_telemetry",
    "config_io_compression_stats",
    "config_imageboot", "config_imageboot_chunklist",
    "config_csr",
    "config_personas",
    "config_proc_uuid_policy",
    "config_coalitions",
    "config_coredump", "config_ucoredump",
    "config_phantom_cache",
    "config_deferred_reclaim",
    "config_mca", "config_mtrr", "config_vmx",
    "config_mbuf_mcache",
    "config_memorystatus",
    "config_requires_u32_munging",
    "content_filter",
    "packet_mangler",
    "remote_vif",
    "fs_compression",
    "quota",
    "kdebug",
    "sysv_sem", "sysv_msg", "sysv_shm",
    "config_mach_bridge_send_time",
    "hypervisor",
    "config_sysdiagnose",
    "config_pv_ticket",
    "zlib", "zlibc",
    "config_iotrace",
    "config_exclaves",
    "config_netboot",
    "nfsserver",
    "nullfs", "bindfs", "mockfs",
    "config_triggers",
    "diagnostic", "profiling",
    "config_ecc_logging",
    "pal_i386",
    "mig_debug",
    "config_kpc",
    "config_sleep",
    "vlan", "bond",
    "if_fake", "if_headless", "if_redirect", "if_bridge", "bridgestp", "gif",
    "necp", "ipsec", "stf",
    "multipath", "mptcp",
    "dummynet",
    "sendfile",
    "pf", "pflog",
    "fdesc", "fifo", "devfs", "routefs",
    "ipv6send",
    "ether", "loop",
    "bpfilter",
    "ptmx", "pty",
    "fb",
    "inet",
    "config_clutch",
    "iokit", "iokitcpp", "libkerncpp",
    "sockets", "networking",
    "hibernation",  # too complex - deal separately
    "kperf", "config_kpc",
    "crypto",
    "mach_kdp", "mach_bsd",
    "debug",
    "skywalk",
    "ipsec",
    "if_loop",
}

# Recompute: Only treat as enabled what is explicitly in ENABLED but not in DISABLED.
ENABLED = ENABLED - DISABLED

def parse(path):
    sources = []
    with open(path) as f:
        for line in f:
            line = line.split("#")[0].strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) < 2:
                continue
            src = parts[0]
            if src.startswith("OPTIONS/"):
                continue
            kind = parts[1]
            if kind == "standard":
                sources.append(src)
            elif kind == "optional":
                # all remaining non-keyword tokens are required features (AND'd)
                feats = [t for t in parts[2:] if t not in ("xnu-library", "bound-checks", "bound-checks-pending")]
                if feats and all(f in ENABLED for f in feats):
                    sources.append(src)
    return sources

if __name__ == "__main__":
    for p in sys.argv[1:]:
        for s in parse(p):
            print(s)
