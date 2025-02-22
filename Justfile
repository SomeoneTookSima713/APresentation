file := if os_family() == "windows" { "apresentation.exe" } else { "apresentation" }

default:
    just --list

build-dev *ARGS:
    cargo build {{ARGS}}

build-release *ARGS:
    cargo build {{ARGS}} --release

run-dev *ARGS: (build-dev ARGS) (setup_running_env)
    cd run/ && ../target/debug/{{file}}

run-release *ARGS: (build-release ARGS) (setup_running_env)
    cd run/ && ../target/release/{{file}}

setup_running_env:
    mkdir -p run/