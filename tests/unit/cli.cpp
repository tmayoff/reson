#include <boost/test/unit_test.hpp>
#include <reson/cli.hpp>

BOOST_AUTO_TEST_SUITE(cli)

BOOST_AUTO_TEST_CASE(Builder) {
  utils::CliBuilder builder{};

  const auto cli = builder.build();
}

BOOST_AUTO_TEST_SUITE_END();
