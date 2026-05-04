load("//bazel:toolchain.bzl", "cc_toolchain_config")
load("@rules_cc//cc:defs.bzl", "cc_toolchain")

package(default_visibility = ["//visibility:public"])

exports_files(["genassym.py"])

filegroup(name = "empty")

cc_toolchain_config(name = "xnu_toolchain_config")

cc_toolchain(
    name = "xnu_cc_toolchain",
    toolchain_identifier = "xnu-toolchain",
    toolchain_config = ":xnu_toolchain_config",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 0,
)

toolchain(
    name = "xnu_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:macos",
        "@platforms//cpu:x86_64",
    ],
    toolchain = ":xnu_cc_toolchain",
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)
