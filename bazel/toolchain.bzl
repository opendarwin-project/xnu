load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load(
    "@rules_cc//cc:cc_toolchain_config_lib.bzl",
    "feature",
    "flag_group",
    "flag_set",
    "tool_path",
)

def _impl(ctx):
    tool_paths = [
        tool_path(
            name = "gcc",
            path = "/usr/bin/clang",
        ),
        tool_path(
            name = "ld",
            path = "/usr/bin/ld.lld",
        ),
        tool_path(
            name = "ar",
            path = "/usr/bin/llvm-libtool-darwin",
        ),
        tool_path(
            name = "cpp",
            path = "/bin/false",
        ),
        tool_path(
            name = "gcov",
            path = "/bin/false",
        ),
        tool_path(
            name = "nm",
            path = "/usr/bin/llvm-nm",
        ),
        tool_path(
            name = "objdump",
            path = "/usr/bin/llvm-objdump",
        ),
        tool_path(
            name = "strip",
            path = "/usr/bin/llvm-strip",
        ),
    ]

    features = [
        feature(
            name = "default_compile_flags",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = [
                        ACTION_NAMES.assemble,
                        ACTION_NAMES.preprocess_assemble,
                        ACTION_NAMES.linkstamp_compile,
                        ACTION_NAMES.c_compile,
                        ACTION_NAMES.cpp_compile,
                        ACTION_NAMES.cpp_header_parsing,
                        ACTION_NAMES.cpp_module_compile,
                        ACTION_NAMES.cpp_module_codegen,
                        ACTION_NAMES.lto_backend,
                        ACTION_NAMES.clif_match,
                    ],
                    flag_groups = [
                        flag_group(
                            flags = [
                                "-target", "x86_64-apple-darwin",
                                "-nostdlibinc",
                            ],
                        ),
                    ],
                ),
            ],
        ),
        feature(
            name = "default_linker_flags",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = [ACTION_NAMES.cpp_link_executable, ACTION_NAMES.cpp_link_dynamic_library, ACTION_NAMES.cpp_link_nodeps_dynamic_library],
                    flag_groups = ([
                        flag_group(
                            flags = [
                                "-target", "x86_64-apple-darwin",
                                "-fuse-ld=lld",
                            ],
                        ),
                    ]),
                ),
            ],
        ),
    ]

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        features = features,
        cxx_builtin_include_directories = ["/usr/lib/clang/22/include"],
        toolchain_identifier = "xnu-toolchain",
        host_system_name = "local",
        target_system_name = "x86_64-apple-darwin",
        target_cpu = "x86_64",
        target_libc = "macosx",
        compiler = "clang",
        abi_version = "unknown",
        abi_libc_version = "unknown",
        tool_paths = tool_paths,
    )

cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {},
)
