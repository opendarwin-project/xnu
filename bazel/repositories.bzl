load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _bootstrap_cmds_extension_impl(ctx):
    http_archive(
        name = "com_apple_oss_bootstrap_cmds",
        url = "https://github.com/apple-oss-distributions/bootstrap_cmds/archive/refs/tags/bootstrap_cmds-138.tar.gz",
        strip_prefix = "bootstrap_cmds-bootstrap_cmds-138",
        build_file = "//bazel:bootstrap_cmds.BUILD",
        sha256 = "7f86f67c13be04504679170e923c6079e706d57b295f10db752f2f8245f7df8c",
    )

bootstrap_cmds_extension = module_extension(implementation = _bootstrap_cmds_extension_impl)
