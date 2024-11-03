#include "cli.hpp"

namespace utils {

// auto Cli::parse(int argc, char** argv) -> void {}

auto CliBuilder::add_command(const std::string& name) -> CliBuilder& {
  return *this;
}

auto CliBuilder::build() -> Cli {
  Cli cli;

  return cli;
}

}  // namespace utils
