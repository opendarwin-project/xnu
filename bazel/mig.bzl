load("@rules_cc//cc:defs.bzl", "cc_common", "CcInfo", "cc_library")
load("@rules_cc//cc:find_cc_toolchain.bzl", "find_cc_toolchain", "use_cc_toolchain")

def _mig_library_impl(ctx):
    cc_toolchain = find_cc_toolchain(ctx)
    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
    )
    cpp_path = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = "c-compile",
    )

    defs_file = ctx.file.src
    base_name = defs_file.basename
    if base_name.endswith(".defs"):
        base_name = base_name[:-5]

    # Preprocessing step
    preprocessed_defs = ctx.actions.declare_file(base_name + ".preprocessed.defs")

    # Outputs
    outputs = []
    mig_args = []

    def handle_output(attr_val, suffix, flag):
        if attr_val == "/dev/null":
            mig_args.extend([flag, "/dev/null"])
            return None
        elif attr_val:
            out = ctx.actions.declare_file(attr_val)
            outputs.append(out)
            mig_args.extend([flag, out.path])
            return out
        else:
            mig_args.extend([flag, "/dev/null"])
            return None

    user_c = handle_output(ctx.attr.user_c, "User.c", "-user")
    server_c = handle_output(ctx.attr.server_c, "Server.c", "-server")
    user_h = handle_output(ctx.attr.user_h, ".h", "-header")
    server_h = handle_output(ctx.attr.server_h, "Server.h", "-sheader")

    # Include paths for preprocessing
    includes = []
    for d in ctx.attr.include_dirs:
        includes.append("-I" + d)
    includes += ["-I.", "-Iosfmk", "-Iosfmk/mach", "-Iosfmk/mach_debug", "-IEXTERNAL_HEADERS", "-Ibsd"]

    # Preprocessor command
    ctx.actions.run_shell(
        inputs = depset(
            [defs_file] + ctx.files.deps,
            transitive = [cc_toolchain.all_files],
        ),
        outputs = [preprocessed_defs],
        command = '(echo "#line 1 \\"{src}\\""; cat "{src}") | {cpp} -E {cpp_flags} {includes} - > {out}'.format(
            src = defs_file.path,
            cpp = cpp_path,
            cpp_flags = " ".join(ctx.attr.cpp_flags),
            includes = " ".join(includes),
            out = preprocessed_defs.path,
        ),
        mnemonic = "MigPreprocess",
        env = cc_common.get_environment_variables(
            feature_configuration = feature_configuration,
            action_name = "c-compile",
            variables = cc_common.create_compile_variables(
                feature_configuration = feature_configuration,
                cc_toolchain = cc_toolchain,
            ),
        ),
    )

    mig_args += ctx.attr.mig_flags

    ctx.actions.run_shell(
        inputs = [preprocessed_defs],
        tools = [ctx.executable._migcom],
        outputs = outputs,
        command = "{migcom} {args} < {input}".format(
            migcom = ctx.executable._migcom.path,
            args = " ".join(mig_args),
            input = preprocessed_defs.path,
        ),
        mnemonic = "MigCom",
    )

    hdrs = [h for h in [user_h, server_h] if h]
    return [
        DefaultInfo(files = depset(outputs)),
        CcInfo(
            compilation_context = cc_common.create_compilation_context(
                headers = depset(hdrs),
                includes = depset([h.dirname for h in hdrs]),
            ),
        ),
    ]

mig_library_rule = rule(
    implementation = _mig_library_impl,
    attrs = {
        "src": attr.label(allow_single_file = [".defs"], mandatory = True),
        "deps": attr.label_list(allow_files = [".defs", ".h"]),
        "cpp_flags": attr.string_list(default = ["-D__MACH30__"]),
        "mig_flags": attr.string_list(),
        "include_dirs": attr.string_list(),
        "user_c": attr.string(),
        "server_c": attr.string(),
        "user_h": attr.string(),
        "server_h": attr.string(),
        "_migcom": attr.label(
            default = Label("@com_apple_oss_bootstrap_cmds//:migcom"),
            executable = True,
            cfg = "exec",
        ),
    },
    fragments = ["cpp"],
    toolchains = use_cc_toolchain(),
    outputs = {
        "user_c": "%{user_c}",
        "server_c": "%{server_c}",
        "user_h": "%{user_h}",
        "server_h": "%{server_h}",
    },
)

def mig_library(name, src, deps = [], cpp_flags = ["-D__MACH30__"], mig_flags = [], include_dirs = [], visibility = None, copts = [], cc_deps = ["//:common_headers"], user_c = None, server_c = None, user_h = None, server_h = None):
    # Default outputs if none specified
    if user_c == None and server_c == None and user_h == None and server_h == None:
        base_name = src.split("/")[-1]
        if base_name.endswith(".defs"):
            base_name = base_name[:-5]
        user_c = base_name + "User.c"
        server_c = base_name + "Server.c"
        user_h = base_name + ".h"
        server_h = base_name + "Server.h"

    mig_library_rule(
        name = name + "_gen",
        src = src,
        deps = deps,
        cpp_flags = cpp_flags,
        mig_flags = mig_flags,
        include_dirs = include_dirs,
        user_c = user_c,
        server_c = server_c,
        user_h = user_h,
        server_h = server_h,
    )

    cc_library(
        name = name,
        srcs = [name + "_gen"],
        hdrs = [name + "_gen"],
        copts = copts,
        deps = cc_deps,
        visibility = visibility,
    )
