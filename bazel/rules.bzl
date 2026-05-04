def genassym(name, src, out, copts = [], includes = [], hdrs = [], deps = []):
    inc_args = ["-I" + i for i in includes]
    # Add root and build roots for generated headers
    inc_args += ["-I.", "-I$(GENDIR)", "-I$(BINDIR)"]
    inc_args += ["-I$(GENDIR)/build_config", "-I$(BINDIR)/build_config"]

    # We use a genrule to run the python script with clang
    native.genrule(
        name = name,
        srcs = [src] + hdrs,
        outs = [out],
        tools = ["//bazel:genassym.py"],
        cmd = "clang " + " ".join(copts) + " " + " ".join(inc_args) + " -I$(GENDIR)/.. -I$(BINDIR)/.. -S -fno-integrated-as -o $@_tmp -- $(location " + src + ") && " +
              "python3 $(location //bazel:genassym.py) clang " + " ".join(copts) + " " + " ".join(inc_args) + " -I$(GENDIR)/.. -I$(BINDIR)/.. -S -fno-integrated-as -- $(location " + src + ") $@",
    )


def dummy_header(name, out, define = None):
    content = ""
    if define:
        content = "#define %s 0\\n" % define

    native.genrule(
        name = name,
        outs = [out],
        cmd = "echo -e '%s' > $@" % content,
    )
