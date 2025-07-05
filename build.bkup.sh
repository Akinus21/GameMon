#!/bin/bash

# Define the output file
OUTPUT_FILE="diagnostic.txt"

# Clear the output file before starting
> "$OUTPUT_FILE"

# Set environment variables (modify as needed)
export RUST_BACKTRACE=1

# Function to run a cargo build and log output
run_build() {
    local platform=$1
    local target=$2
    local binary_name=$3
    local label=$4

    # Construct the build command
    if [[ "$platform" == "windows" ]]; then
        build_cmd="cargo xwin build --release --target $target --bin $binary_name --features windows"
    else
        build_cmd="cargo build --release --target $target --bin $binary_name --features linux"
        cp ./target/x86_64-unknown-linux-gnu/release/$binary_name /home/gabriel/.local/share/gamemon/$binary_name
    fi

    # Run the command and check for success/failure
    echo "[INFO] Running: $build_cmd" | tee -a "$OUTPUT_FILE"
    if eval "$build_cmd" &>> "$OUTPUT_FILE"; then
        echo "[SUCCESS] $label built successfully!" | tee -a "$OUTPUT_FILE"
        echo "..................................................................................................................................." >> "$OUTPUT_FILE"
        echo ".  ^^ $label" >> "$OUTPUT_FILE"
        echo "..................................................................................................................................." >> "$OUTPUT_FILE"
        echo "" | tee -a "$OUTPUT_FILE"
    else
        echo "[ERROR] $label failed to build. Check $OUTPUT_FILE for details." | tee -a "$OUTPUT_FILE"
        echo "..................................................................................................................................." >> "$OUTPUT_FILE"
        echo ".  ^^ $label" >> "$OUTPUT_FILE"
        echo "..................................................................................................................................." >> "$OUTPUT_FILE"
        echo "" | tee -a "$OUTPUT_FILE"
    fi

    echo "" >> "$OUTPUT_FILE"
}

# Default: Build for all platforms
build_windows=false
build_linux=false

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --windows)
            build_windows=true
            shift
            ;;
        --linux)
            build_linux=true
            shift
            ;;
        --all)
            build_windows=true
            build_linux=true
            shift
            ;;
        *)
            echo "Usage: $0 [--windows] [--linux] [--all]"
            exit 1
            ;;
    esac
done

# If no specific platform was selected, build all by default
if ! $build_windows && ! $build_linux; then
    build_windows=true
    build_linux=true
fi

# Run builds for Linux
if $build_linux; then
    unset PKG_CONFIG_PATH
    unset PKG_CONFIG_SYSROOT_DIR
    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-service" "Linux Service"
    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-gui" "Linux GUI"
    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-update" "Linux Update"
fi

# Run builds for Windows
if $build_windows; then
    export PKG_CONFIG_SYSROOT_DIR=/usr/x86_64-w64-mingw32/sys-root
    export PKG_CONFIG_PATH=/usr/x86_64-w64-mingw32/sys-root/mingw/lib/pkgconfig
    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-service" "Windows Service"
    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-gui" "Windows GUI"
    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-update" "Windows Update"

fi

echo "Build process complete. See '$OUTPUT_FILE' for details."
unset PKG_CONFIG_PATH
unset PKG_CONFIG_SYSROOT_DIR

