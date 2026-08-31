init_bin = executable(
    'init',
    files('init/init.c'),
    c_args: [
        '-target',
        target_triple,
        '-ffreestanding',
        '-nostdlib',
    ],
    link_args: [
        '-target',
        target_triple,
        '-nostdlib',
        '-static',
        '-e',
        '__start',
    ],
    pie: false,
    install: false,
)
