tests = ['cli']
foreach test : tests
    test(
        'reson_utils_@0@'.format(test),
        executable(
            'reson_utils_@0@_test'.format(test),
            '@0@.cpp'.format(test),
            dependencies: [reson_dep, boost_test],
            cpp_args: ['-DBOOST_TEST_MODULE=@0@'.format(test)],
        ),
    )
endforeach
