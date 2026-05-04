load("@rules_cc//cc:defs.bzl", "cc_binary")

genrule(
    name = "lex_yy_c",
    srcs = ["migcom.tproj/lexxer.l"],
    outs = ["lex.yy.c"],
    cmd = "flex -o $@ $<",
)

genrule(
    name = "parser_y_c",
    srcs = ["migcom.tproj/parser.y"],
    outs = [
        "y.tab.c",
        "y.tab.h",
    ],
    cmd = "bison -y -d -o $(location y.tab.c) $<",
)

cc_binary(
    name = "migcom",
    srcs = glob(
        [
            "migcom.tproj/*.c",
            "migcom.tproj/*.h",
        ],
        exclude = ["migcom.tproj/handler.c"],
    ) + [
        ":lex_yy_c",
        ":parser_y_c",
        "@//bazel/mig_compat:compat.h",
        "@//bazel/mig_compat:mach/boolean.h",
        "@//bazel/mig_compat:mach/message.h",
        "@//bazel/mig_compat:mach/kern_return.h",
        "@//bazel/mig_compat:mach/std_types.h",
        "@//bazel/mig_compat:mach/ndr.h",
    ],
    includes = ["migcom.tproj"],
    copts = [
        "-Ibazel/mig_compat",
        "-include bazel/mig_compat/compat.h",
        "-DYY_NO_UNPUT",
        "-DYY_NO_INPUT",
    ],
    visibility = ["//visibility:public"],
)
