file := if os_family() == "windows" { "apresentation.exe" } else { "apresentation" }

default:
    just --list

build-dev *ARGS:
    cargo build {{ARGS}}

build-release *ARGS:
    cargo build {{ARGS}} --release

run-dev *ARGS: (build-dev (replace_regex(ARGS, "--\\s.+", ""))) (setup_running_env)
    cd run/ && ../target/debug/{{file}} {{ replace_regex(ARGS, ".*--\\s", "") }}

run-release *ARGS: (build-release ARGS) (setup_running_env)
    cd run/ && ../target/release/{{file}}

setup_running_env:
    mkdir -p run/