#pragma once

#include <string>

namespace utils {

class Cli {
 public:
  auto parse();
};

class CliBuilder {
 public:
  auto add_command(const std::string& name) -> CliBuilder&;
  auto add_option(const std::string& name) -> CliBuilder&;

  auto build() -> Cli;

 private:
};

}  // namespace utils
