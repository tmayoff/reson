#pragma once

#include <span>
#include <string>

namespace utils {

class Cli {
 public:
  // auto parse(int argc, char** argv) -> void;
  auto parse(auto args) { args_ = std::span(args.begin(), args.end()); }

 private:
  std::span<std::string_view> args_;
};

class CliBuilder {
 public:
  auto add_command(const std::string& name) -> CliBuilder&;
  auto add_option(const std::string& name) -> CliBuilder&;

  auto build() -> Cli;

 private:
  struct Command {
    std::string name;
    // Command subcommand;
  };

  Command command_;
};

}  // namespace utils
