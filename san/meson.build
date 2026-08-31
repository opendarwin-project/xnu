# Sanitizer stubs / support if needed
san_sources = files()
if san_sources.length() > 0
    xnu_san = static_library(
        'xnu_san',
        san_sources,
        c_args: comp_cflags['san'] + c_args,
        kwargs: lib_kwargs,
    )
    kernel_libs += [xnu_san]
endif
