#include <boost/test/unit_test.hpp>
#include <reson/cli.hpp>

BOOST_AUTO_TEST_SUITE(cli)

BOOST_AUTO_TEST_CASE(Builder) {
  auto cli = utils::CliBuilder{}.add_command("setup").build();

  std::vector<std::string_view> argv = {"help"};

  cli.parse(argv);
}

BOOST_AUTO_TEST_SUITE_END();
