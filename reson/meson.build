reson_lib = static_library(
    'reson',
    'src/cli.cpp',
    include_directories: ['include', 'include/reson'],
)
reson_dep = declare_dependency(
    include_directories: 'include',
    link_with: reson_lib,
)

reson = executable('reson', 'src/main.cpp', dependencies: reson_dep)
