alias build := build-debug
alias run := run-debug

# Builds the binary in debug mode
build-debug:
    -cargo build
    @rm src/version

# Builds the binary in release mode
build-release:
    -cargo build --release
    @rm src/version

# Builds and runs the project in debug mode
run-debug CMD PATH:
    -cargo run -- {{CMD}} {{PATH}}
    @rm src/version

# Builds and runs the project in release mode
run-release CMD PATH:
    -cargo run --release -- {{CMD}} {{PATH}}
    @rm src/version

# Cleans up any temporary files
cleanup:
    rm src/version
    rm src/OpenSans.ttf