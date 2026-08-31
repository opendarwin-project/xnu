pexpert_c_sources = files(
    'gen/device_tree.c',
    'gen/bootargs.c',
    'gen/pe_gen.c',
    'gen/kcformat.c',
    'arm/pe_bootargs.c',
    'arm/pe_consistent_debug.c',
    'arm/pe_fiq.c',
    'arm/pe_identify_machine.c',
    'arm/pe_init.c',
    'arm/pe_kprintf.c',
    'arm/pe_serial.c',
    'arm/hwtrace/hwtrace.c',
    'arm/hwtrace/apt_msg.c',
)
icu_source = files(board['icu_source'])

pexpert_cflags = comp_cflags['pexpert'] + c_args

xnu_pexpert = static_library(
    'xnu_pexpert',
    pexpert_c_sources + icu_source + gen_order,
    c_args: pexpert_cflags,
    kwargs: lib_kwargs,
)

kernel_libs += [xnu_pexpert]
