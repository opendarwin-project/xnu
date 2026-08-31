libsa_cpp_sources = files(
    'bootstrap.cpp',
)

libsa_cflags = comp_cflags['libsa'] + c_args
libsa_cxxflags = comp_cflags['libsa'] + cxx_args

xnu_libsa = static_library(
    'xnu_libsa',
    libsa_cpp_sources + gen_order,
    c_args: libsa_cflags,
    cpp_args: libsa_cxxflags,
    kwargs: lib_kwargs,
)

kernel_libs += [xnu_libsa]
