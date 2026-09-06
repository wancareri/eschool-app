#!/usr/bin/env bash
set -euo pipefail

REPO="kayzer4/eschool-app"
DIST_DIR="dist"

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [PLATFORM...]

Download build artifacts from GitHub Actions.

Platforms:
  ios        iOS IPA (unsigned)
  macos      macOS DMG
  android    Android APK
  linux      Linux AppImage
  windows    Windows MSIX
  all        All platforms

Options:
  -l, --list       List available artifacts
  -d, --dir DIR    Output directory (default: $DIST_DIR)
  -h, --help       Show this help

Examples:
  $(basename "$0") ios macos       # Download iOS + macOS
  $(basename "$0") all             # Download everything
  $(basename "$0") -l              # List available artifacts
EOF
}

list_artifacts() {
    echo "Available artifacts for $REPO:"
    echo "---"
    gh api "repos/$REPO/actions/runs?per_page=1&status=success" \
        --jq '.workflow_runs[0] | "Run: \(.head_sha[0:7]) (\(.created_at))"' 2>/dev/null || true

    gh api "repos/$REPO/actions/runs?per_page=1&status=success/artifacts" 2>/dev/null \
        --jq '.artifacts[] | "  \(.name) (\(.size_in_bytes / 1048576 | round)MB)"' 2>/dev/null || {
        # Fallback: get latest successful run artifacts
        local run_id
        run_id=$(gh api "repos/$REPO/actions/runs?per_page=10&status=success" \
            --jq '.workflow_runs[] | select(.conclusion == "success") | .id' 2>/dev/null | head -1)
        if [[ -n "$run_id" ]]; then
            gh api "repos/$REPO/actions/runs/$run_id/artifacts" \
                --jq '.artifacts[] | "  \(.name) (\(.size_in_bytes / 1048576 | round)MB)"' 2>/dev/null
        fi
    }
}

download_artifact() {
    local name="$1"
    local output="$2"

    echo "Downloading $name..."
    gh api "repos/$REPO/actions/runs?per_page=10&status=success" \
        --jq '.workflow_runs[] | select(.conclusion == "success") | .id' 2>/dev/null | \
    while read -r run_id; do
        if gh api "repos/$REPO/actions/runs/$run_id/artifacts" \
            --jq ".artifacts[] | select(.name == \"$name\") | .archive_download_url" 2>/dev/null | \
           head -1 | grep -q .; then
            local url
            url=$(gh api "repos/$REPO/actions/runs/$run_id/artifacts" \
                --jq ".artifacts[] | select(.name == \"$name\") | .archive_download_url" 2>/dev/null | head -1)
            gh api "$url" > "$output" 2>/dev/null
            echo "  -> $output"
            return 0
        fi
    done
    echo "  ! Artifact '$name' not found"
    return 1
}

PLATFORMS=()
OUTPUT_DIR="$DIST_DIR"
LIST_ONLY=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -l|--list) LIST_ONLY=true; shift ;;
        -d|--dir) OUTPUT_DIR="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        all) PLATFORMS+=(ios macos android linux windows); shift ;;
        ios|macos|android|linux|windows) PLATFORMS+=("$1"); shift ;;
        *) echo "Unknown: $1"; usage; exit 1 ;;
    esac
done

if ! command -v gh &>/dev/null; then
    echo "Error: 'gh' CLI not found. Install: https://cli.github.com/"
    exit 1
fi

if ! gh auth status &>/dev/null 2>&1; then
    echo "Error: Not authenticated. Run: gh auth login"
    exit 1
fi

if $LIST_ONLY; then
    list_artifacts
    exit 0
fi

if [[ ${#PLATFORMS[@]} -eq 0 ]]; then
    usage
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

declare -A ARTIFACT_MAP=(
    [ios]="ios-ipa"
    [macos]="macos-dmg"
    [android]="android-apk"
    [linux]="linux-appimage"
    [windows]="windows-exe"
)

declare -A EXT_MAP=(
    [ios]="ipa"
    [macos]="dmg"
    [android]="apk"
    [linux]="AppImage"
    [windows]="msix"
)

for platform in "${PLATFORMS[@]}"; do
    artifact_name="${ARTIFACT_MAP[$platform]}"
    ext="${EXT_MAP[$platform]}"
    output="$OUTPUT_DIR/eschool-${platform}.${ext}"
    download_artifact "$artifact_name" "$output" || true
done

echo ""
echo "Done. Files in $OUTPUT_DIR/:"
ls -lh "$OUTPUT_DIR/" 2>/dev/null
