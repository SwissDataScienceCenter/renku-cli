#!/usr/bin/env bash

set -euo pipefail

os=$(uname)
arch=$(uname -m)
dist=$(uname -o)
api_url_prefix="${API_URL_PREFIX:-https://api.github.com/repos/SwissDataScienceCenter/renku-cli/releases}"
dl_url_prefix="${DL_URL_PREFIX:-https://github.com/SwissDataScienceCenter/renku-cli/releases}"
binary_name="${BINARY_NAME:-rnk}"
version=""
check_latest=0
verbosity=${VERBOSITY:-0}
jq=""
tdir="."
target="/usr/local/bin"
os_id=""

if type -P mktemp > /dev/null; then
    tdir=$(mktemp -d -t renku-cli-install.XXXXXXX)
fi

trap cleanup 1 2 3 6 ERR


debug() {
    printf "%s\n" "$*" >&2
}

debug_v() {
    [[ $verbosity -eq 0 ]] || debug "$*"
}

cleanup() {
    if ! [ "$tdir" == "." ]; then
        debug_v "Cleanup temporary files: $tdir"
        rm -rf "$tdir"
    fi
    exit
}

assert_exec() {
    local program="$1"
    if ! type -P "$1" >/dev/null; then
        debug "$1 is not installed. Please install $1 and run this script again."
        exit 1
    fi
}

curl_silent() {
    if [ "$verbosity" -eq 0 ]; then
        curl -sSL -o /dev/null --stderr /dev/null --fail "$@"
    else
        debug "curl -sSL --fail $@"
        curl -sSL -o /dev/null --fail "$@"
    fi
}

print_help() {
    debug "Install script for renku-cli for macos and linux"
    debug
    debug "Usage: $(basename "$0") [-h][-c][-v <version>]"
    debug "Options:"
    debug "  -h   print this help"
    debug "  -c   check for latest version"
    debug "  -v   verbose mode"
    debug "  -t <tag_name>"
    debug "       Install the given version instead of latest"
}

find_latest_release() {
    # get latest release
    local latest_response=$(curl -sSL "$api_url_prefix/latest")
    local version=$(echo $latest_response | jq -r '.tag_name')
    local version_num="${version:1}"

    local suffix=""
    case "$os" in
        Linux)
        ;;
        Darwin)
            suffix="darwin-"
            ;;
        *)
            debug "Unknown os: $os"
            exit 1
    esac

    case "$arch" in
        x86_64)
            suffix="${suffix}amd64"
            ;;
        aarch64)
            suffix="${suffix}aarch64"
            ;;
        arm64)
            suffix="${suffix}aarch64"
            ;;
        *)
            debug "Unknown architecture: $arch"
            exit 1
    esac

    if [ "$os_id" == "alpine" ]; then
        suffix="${suffix}-musl"
    fi

    local name_prefix="rnk_${suffix}-${version_num}"
    local url=$(echo $latest_response | jq -r ".assets[]|select(.name | startswith(\"$name_prefix\"))|.browser_download_url")
    if [ -z "$url" ]; then
        echo "No download url could be found for $name_prefix."
        exit 1
    fi
    echo $version_num $url
}

while getopts "ht:cv" arg; do
    case $arg in
        h)
            print_help
            exit 0
            ;;
        v)
            verbosity=1
            ;;
        t)
            version=$OPTARG
            debug_v "Set version: $version"
            ;;
        c)
            check_latest=1
            ;;
    esac
done

# The aarch64 executables won't work on Android
if [ "$dist" == "Android" ]; then
    debug "Sorry, Android is not yet supported."
    exit 1
fi

## check for curl
assert_exec "curl"
## check for jq
assert_exec jq
## check for cut, grep (should be available)
assert_exec "cut"
assert_exec "grep"

if [ -r /etc/os-release ]; then
    os_id=$(cat /etc/os-release | grep "^ID=" | cut -d'=' -f2 | tr -d '[:space:]')
fi

if [ $check_latest -eq 1 ]; then
    debug_v "Check for latest version only"
    find_latest_release
    exit 0
else
    # Check for nixos
    if [ "$os_id" == "nixos" ]; then
        debug "For NixOS, please install via:"
        debug "  nix profile install github:SwissDataScienceCenter/renku-cli"
        debug "or use the flake in your nixos configuration."
        debug "See https://github.com/SwissDataScienceCenter/renku-cli/blob/main/docs/install.md#nix"
    else
        ## check for sudo first
        assert_exec "sudo"

        read version url < <(find_latest_release)
        debug "Getting renku-cli $version ..."
        debug_v "from: $url"
        curl -# -sSL --fail -o "$tdir/rnk" "$url"
        chmod 755 "$tdir/rnk"

        debug "Installing to $target"
        sudo mkdir -p "$target"
        sudo cp "$tdir/rnk" "$target/rnk"
        debug "Done."
    fi
fi

cleanup
