#!/bin/bash
set -euo pipefail

# Define the output file
OUTPUT_FILE="diagnostic.txt"

# Clear the output file before starting
: > "$OUTPUT_FILE"

# Set environment variables (modify as needed)
export RUST_BACKTRACE=1

# Project root
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASES_DIR="$PROJECT_DIR/releases"
mkdir -p "$RELEASES_DIR"

# Logging functions
log_step() {
    local message="$1"
    printf "\r\033[K\r[INFO] %s..." "$message"
    echo "[INFO] $message..." >> "$OUTPUT_FILE"
}

log_step_done() {
    local message="$1"
    printf "\r\r\033[K[SUCCESS] %s\n" "$message"
    echo "[SUCCESS] $message" >> "$OUTPUT_FILE"
}

log_step_fail() {
    local message="$1"
    printf "\r\r\033[K[ERROR] %s\n" "$message"
    echo "[ERROR] $message" >> "$OUTPUT_FILE"
}

log_info_block() {
    local message="$1"
    printf "\n[INFO] %s\n" "$message"
    echo "[INFO] $message" >> "$OUTPUT_FILE"
}

log_complete_block() {
    local message="$1"
    printf "\n[COMPLETE] %s\n" "$message"
    echo "[COMPLETE] $message" >> "$OUTPUT_FILE"
}

# Function to update Cargo.toml version if needed
# This function checks the current version in Cargo.toml and updates it if it doesn't match NEW
update_cargo_version_if_needed() {
    local current_version
    current_version=$(awk -F '=' '/^version/ {gsub(/"/, "", $2); print $2; exit}' "$PROJECT_DIR/Cargo.toml")

    if [[ "$NEW_VERSION" != "$current_version" ]]; then
        log_step "Updating Cargo.toml version from $current_version to $NEW_VERSION"
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$PROJECT_DIR/Cargo.toml"
        log_step_done "Cargo.toml version updated"
    else
        log_step "Cargo.toml version already matches $NEW_VERSION"
        echo "[INFO] Cargo.toml version already matches $NEW_VERSION" >> "$OUTPUT_FILE"
    fi
}

# Function to run a cargo build and log output
run_build() {
    local platform=$1
    local target=$2
    local binary_name=$3
    local label=$4
    local build_cmd

    if [[ "$platform" == "windows" ]]; then
        build_cmd="cargo xwin build --release --target $target --bin $binary_name --features windows"
    else
        build_cmd="cargo build --release --target $target --bin $binary_name --features linux"
    fi

    log_step "Building $label ($target)"

    if eval "$build_cmd" &>> "$OUTPUT_FILE"; then
        log_step_done "$label built successfully!"
        echo ".........................................................................................................." >> "$OUTPUT_FILE"
        echo ".  ^^ $label" >> "$OUTPUT_FILE"
        echo ".........................................................................................................." >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    else
        log_step_fail "$label failed to build. Check $OUTPUT_FILE for details."
        echo ".........................................................................................................." >> "$OUTPUT_FILE"
        echo ".  ^^ $label" >> "$OUTPUT_FILE"
        echo ".........................................................................................................." >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
}

# Function to parse multi-line sections from .new-release
parse_new_release() {
    if [[ ! -f .new-release ]]; then
        log_step_fail ".new-release file not found!"
        return 1
    fi

    TAGS=$(awk '/^Tags:/ {getline; print}' .new-release || echo "")
    if [[ -z "$TAGS" ]]; then
        TAGS=$(awk -F '=' '/^version/ {gsub(/"/, "", $2); print $2; exit}' Cargo.toml | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
        log_step "No tags specified in .new-release. Using version from Cargo.toml: $TAGS"
    fi

    # Helper function to extract multi-line section until next header or blank line
    extract_section() {
        awk -v header="$1" '
            $0 ~ "^" header ":" {flag=1; next}
            /^[A-Za-z ]+:/ && flag {flag=0}
            flag && NF {print}
        ' .new-release || echo ""
    }

    BODY=$(extract_section "Message")
    FEATURES=$(extract_section "Features")
    TODOS=$(extract_section "ToDo")
    ISSUES=$(extract_section "Known Issues")
    UPDATES=$(extract_section "Updates")
}

# Function to get crate name
get_crate_name() {
    if [[ ! -f Cargo.toml ]]; then
        log_step_fail "Cargo.toml not found!"
        return 1
    fi
    RAW_NAME=$(awk -F "=" '/^name/ {gsub(/"/, "", $2); print $2; exit}' Cargo.toml | xargs)
    # Replace underscores with spaces, simple title-case
    CRATE_NAME=$(to_title_case "$(echo "$RAW_NAME" | sed -E 's/_/ /g')")
}

# Function to parse .commit file with checks
parse_commit_file() {
    if [[ ! -f ".commit" ]]; then
        log_step_fail ".commit file not found."
        return 1
    fi

    GITHUB_TOKEN=$(awk -F= '/GITHUB_TOKEN/ {gsub(/"/, "", $2); print $2}' .commit)
    GITHUB_REPO=$(awk -F= '/GITHUB_REPO/ {gsub(/"/, "", $2); print $2}' .commit)
    COMMIT_MESSAGE=$(awk -F= '/MESSAGE/ {gsub(/"/, "", $2); print $2}' .commit)
    LAST_MESSAGE=$(awk -F= '/LAST_MESSAGE/ {gsub(/"/, "", $2); print $2}' .commit)
}

# Function to update .commit with last message
update_commit_file_after_commit() {
    sed -i 's/^MESSAGE=.*/MESSAGE=""/' .commit
    sed -i "s/^LAST_MESSAGE=.*/LAST_MESSAGE=\"$COMMIT_MESSAGE\"/" .commit
}

# Function to package release archives
package_release() {
    ASSETS=()
    # Linux archives
    for dir in "$PROJECT_DIR/target"/*-linux*; do
        [[ -d "$dir/release" ]] || continue
        folder=$(basename "$dir")
        out_file="$RELEASES_DIR/GameMon-v${NEW_VERSION}-${folder}.tar.gz"
        tar -czf "$out_file" -C "$dir/release" GameMon-gui GameMon-service GameMon-update -C "$PROJECT_DIR" resources
        log_step_done "Created Linux archive: $out_file"
        ASSETS+=("$out_file")
    done

    # Windows archives
    for dir in "$PROJECT_DIR/target"/*-windows*; do
        [[ -d "$dir/release" ]] || continue
        folder=$(basename "$dir")
        out_file="$RELEASES_DIR/GameMon-v${NEW_VERSION}-${folder}.zip"
        (cd "$dir/release" && zip -r "$out_file" GameMon-gui.exe GameMon-service.exe GameMon-update.exe) >> "$OUTPUT_FILE" 2>&1
        (cd "$PROJECT_DIR" && zip -ur "$out_file" resources) >> "$OUTPUT_FILE" 2>&1
        log_step_done "Created Windows archive: $out_file"
        ASSETS+=("$out_file")
    done
}

to_title_case() {
    echo "$1" | sed -E 's/(^| )([a-z])/\U\2/g'
}

# Function to create GitHub release and upload assets
create_github_release() {
    # Install jq if missing
    if ! command -v jq &> /dev/null; then
        log_step "jq not found, attempting to install..."
        if command -v dnf &> /dev/null; then
            sudo dnf install -y jq >> "$OUTPUT_FILE" 2>&1
        elif command -v apt-get &> /dev/null; then
            sudo apt-get update >> "$OUTPUT_FILE" 2>&1
            sudo apt-get install -y jq >> "$OUTPUT_FILE" 2>&1
        else
            log_step_fail "Could not install jq. Please install it manually."
            exit 1
        fi
    fi

    log_step "Checking for existing GitHub release with tag $TAGS..."

    existing_release=$(curl -s -H "Authorization: Bearer ${GITHUB_TOKEN}" \
        "https://api.github.com/repos/${GITHUB_REPO}/releases/tags/${TAGS}")

    release_id=$(echo "$existing_release" | jq -r '.id')
    if [[ "$release_id" != "null" ]]; then
        log_step "Release with tag '$TAGS' exists. Deleting existing release and tag..."

        curl -s -X DELETE -H "Authorization: Bearer ${GITHUB_TOKEN}" \
            "https://api.github.com/repos/${GITHUB_REPO}/releases/${release_id}" >> "$OUTPUT_FILE" 2>&1

        curl -s -X DELETE -H "Authorization: Bearer ${GITHUB_TOKEN}" \
            "https://api.github.com/repos/${GITHUB_REPO}/git/refs/tags/${TAGS}" >> "$OUTPUT_FILE" 2>&1

        log_step_done "Existing release and tag deleted."
    fi

    RELEASE_BODY="**${BODY}**

**Features:**
${FEATURES}

**ToDo:**
${TODOS}

**Known Issues:**
${ISSUES}

**Updates:**$(echo "$UPDATES" | sed -E 's/^Update[[:space:]]*-(.*)/\
\
*Update -\1*/')"

    ESCAPED_BODY=$(printf '%s' "$RELEASE_BODY" | jq -Rs .)
    ESCAPED_TAGS=$(printf '%s' "$TAGS" | jq -Rs .)
    ESCAPED_NAME=$(printf '%s' "${CRATE_NAME} v${NEW_VERSION}" | jq -Rs .)

    log_step "Creating GitHub release..."
    RESPONSE=$(curl -s -X POST "https://api.github.com/repos/${GITHUB_REPO}/releases" \
        -H "Authorization: Bearer ${GITHUB_TOKEN}" \
        -H "Content-Type: application/json" \
        -d @- <<EOF
{
  "tag_name": $ESCAPED_TAGS,
  "name": $ESCAPED_NAME,
  "body": $ESCAPED_BODY,
  "draft": false,
  "prerelease": false
}
EOF
)

    UPLOAD_URL=$(echo "$RESPONSE" | jq -r .upload_url | sed -e "s/{?name,label}//")

    if [[ "$UPLOAD_URL" == "null" ]]; then
        log_step_fail "Failed to create release."
        echo "$RESPONSE" >> "$OUTPUT_FILE"
        return 1
    fi

    for asset in "${ASSETS[@]}"; do
        filename=$(basename "$asset")
        log_step "Uploading asset: $filename"
        curl -s --data-binary @"$asset" -H "Authorization: Bearer ${GITHUB_TOKEN}" \
            -H "Content-Type: application/octet-stream" \
            "$UPLOAD_URL?name=$filename" >> "$OUTPUT_FILE" 2>&1
    done

    log_step_done "GitHub release created and assets uploaded."
}

# Function to perform git sync
git_sync() {
    if [[ ! -f ".commit" ]]; then
        log_step_fail ".commit file not found. Skipping Git commit."
        return 1
    fi

    parse_commit_file

    if [[ -z "$COMMIT_MESSAGE" ]]; then
        read -rp "Enter commit message: " COMMIT_MESSAGE
    fi

    if [[ "$COMMIT_MESSAGE" == "reuse last" ]]; then
        COMMIT_MESSAGE="$LAST_MESSAGE"
    fi

    cd "$PROJECT_DIR"
    if [[ -n "$(git status --porcelain)" ]]; then
        git add .
        git commit -m "$COMMIT_MESSAGE" >> "$OUTPUT_FILE" 2>&1
        update_commit_file_after_commit
        git push origin main >> "$OUTPUT_FILE" 2>&1
        log_info_block "Git commit and push completed."
    else
        log_info_block "No changes to commit."
    fi
}

# Parse CLI args with defaults
build_windows=false
build_linux=false
new_release=false
git_sync_flag=false
NEW_VERSION=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --windows) build_windows=true; shift;;
        --linux) build_linux=true; shift;;
        --all) build_windows=true; build_linux=true; shift;;
        --new-release)
            create_release=true; shift
            if [[ $# -gt 0 && "$1" != --* ]]; then
                NEW_VERSION="$1"; shift
                update_cargo_version_if_needed
            else
                NEW_VERSION=$(awk -F '=' '/^version/ {gsub(/"/, "", $2); print $2}' "$PROJECT_DIR/Cargo.toml")
                log_step "Using version from Cargo.toml: $NEW_VERSION"
            fi
            ;;
        --git-sync)
            git_sync_flag=true
            shift
            ;;
        *) echo "Usage: $0 [--windows] [--linux] [--all] [--new-release <version>] [--git-sync]"; exit 1;;
    esac
done

# Default: build both if none specified
if ! $build_windows && ! $build_linux; then
    build_windows=true
    build_linux=true
fi

if $build_linux; then
    unset PKG_CONFIG_PATH
    unset PKG_CONFIG_SYSROOT_DIR

    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-service" "Linux Service"
    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-gui" "Linux GUI"
    run_build "linux" "x86_64-unknown-linux-gnu" "GameMon-update" "Linux Update"

    if ! $new_release; then
        log_step "Copying built Linux binaries to local install locations"
        
        pkill -x GameMon-gui || true
        sleep 1
        cp -f "$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/GameMon-gui" "$HOME/.local/share/gamemon/GameMon-gui" || log_step_fail "Failed to copy GameMon-gui"
        
        pkill -x GameMon-service || true
        sleep 1
        cp -f "$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/GameMon-service" "$HOME/.local/share/gamemon/GameMon-service" || log_step_fail "Failed to copy GameMon-service"
        
        pkill -x GameMon-update || true
        sleep 1
        cp -f "$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/GameMon-update" "$HOME/.local/share/gamemon/GameMon-update" || log_step_fail "Failed to copy GameMon-update"
        
        nohup "$HOME/.local/share/gamemon/GameMon-service" &> /dev/null &

        log_step_done "Binaries copied."
    fi
fi

if $build_windows; then
    export PKG_CONFIG_SYSROOT_DIR=/usr/x86_64-w64-mingw32/sys-root
    export PKG_CONFIG_PATH=/usr/x86_64-w64-mingw32/sys-root/mingw/lib/pkgconfig

    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-service" "Windows Service"
    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-gui" "Windows GUI"
    run_build "windows" "x86_64-pc-windows-msvc" "GameMon-update" "Windows Update"
fi

if $new_release; then
    log_step "Creating new GitHub release..."
    parse_new_release
    parse_commit_file
    get_crate_name
    package_release
    create_github_release
fi

if $git_sync_flag; then
    git_sync
fi

log_complete_block "Build process complete. See '$OUTPUT_FILE' for details."

# Clean up env
unset PKG_CONFIG_PATH
unset PKG_CONFIG_SYSROOT_DIR
