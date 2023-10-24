alias build := build-debug
alias run := run-debug

default:
    just --list

# Builds the binary in debug mode
build-debug:
    -cargo build
    @rm src/version

# Builds the binary in release mode
build-release:
    -cargo build --release
    @rm src/version

# Builds the binary in release mode with extra optimizations. It takes *way* longer, but is also more optimized.
build-optimized:
    @echo "WARN: This build will take considerably longer than building in release mode!"
    -cargo build --profile=optimized
    @rm src/version

# Like build-optimized, but may also enable some cpu-specific optimizations for newer CPUs
build-optimized-native $RUSTFLAGS="-C target-cpu=native -Z tune-cpu=znver4":
    @echo "WARN: This build will take considerably longer than building in release mode!"
    -cargo build --profile=optimized
    @rm src/version

# Builds and runs the project in debug mode
run-debug CMD PATH:
    -cargo run -- {{CMD}} {{PATH}}
    @rm src/version

# Builds and runs the project in release mode
run-release CMD PATH:
    -cargo run --release -- {{CMD}} {{PATH}}
    @rm src/version

# Builds and runs the project in release mode with extra optimizations. It takes *way* longer, but is also more optimized.
run-optimized CMD PATH:
    @echo "WARN: This build will take considerably longer than building in release mode!"
    -cargo run --profile=optimized -- {{CMD}} {{PATH}}
    @rm src/version

# Like run-optimized, but may also enable some cpu-specific optimizations for newer CPUs
run-optimized-native CMD PATH $RUSTFLAGS="-C target-cpu=native -Z tune-cpu=znver4":
    @echo "WARN: This build will take considerably longer than building in release mode!"
    -cargo run --profile=optimized -- {{CMD}} {{PATH}}
    @rm src/version

# Cleans up any temporary files
cleanup:
    rm src/version
    rm src/OpenSans.ttf