#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
minimum_supported_release="$(
  sed -n 's/^pub const MINIMUM_SUPPORTED_BD3_VERSION: &str = "\([^"]*\)";$/\1/p' \
    "${repo_root}/bd3-rs/exec-server-protocol/src/lib.rs"
)"
: "${minimum_supported_release:?minimum supported Bd3 release is missing}"
release_directory="$(mktemp -d "${TMPDIR:-/tmp}/bd3-exec-server-skew.XXXXXX")"
trap 'rm -rf "${release_directory:?}"' EXIT

if [[ $# -eq 0 ]]; then
  releases=(latest "${minimum_supported_release}")
else
  releases=("$@")
fi

case "$(uname -s):$(uname -m)" in
  Darwin:arm64) target="aarch64-apple-darwin" ;;
  Darwin:x86_64) target="x86_64-apple-darwin" ;;
  Linux:aarch64 | Linux:arm64) target="aarch64-unknown-linux-musl" ;;
  Linux:x86_64) target="x86_64-unknown-linux-musl" ;;
  *)
    echo "Unsupported platform: $(uname -s) $(uname -m)" >&2
    exit 1
    ;;
esac

asset="bd3-${target}.tar.gz"
cd "${repo_root}/bd3-rs"
cargo build -p bd3-cli --bin 3bd
export BD3_TEST_CURRENT_BD3="${CARGO_TARGET_DIR:-${repo_root}/bd3-rs/target}/debug/bd3"

echo "Testing current Bd3 compatibility through authenticated Noise"
export BD3_TEST_RELEASED_BD3="${BD3_TEST_CURRENT_BD3}"
just test -p bd3-exec-server --test relay version_skew --test-threads 1

tested_release_version=""
for release in "${releases[@]}"; do
  release="${release#rust-v}"
  if [[ "${release}" == "${tested_release_version}" ]]; then
    echo "Skipping Bd3 ${release}; this release was already tested"
    continue
  fi

  if [[ "${release}" == "latest" ]]; then
    release_url="https://github.com/openai/codex/releases/latest/download/${asset}"
  else
    release_url="https://github.com/openai/codex/releases/download/rust-v${release}/${asset}"
  fi

  binary_directory="${release_directory}/${release}"
  mkdir -p "${binary_directory}"
  echo "Downloading released Bd3 from ${release_url}"
  curl -fsSL "${release_url}" -o "${binary_directory}/${asset}"
  tar -xzf "${binary_directory}/${asset}" -C "${binary_directory}"

  export BD3_TEST_RELEASED_BD3="${binary_directory}/bd3-${target}"
  release_output="$("${BD3_TEST_RELEASED_BD3}" --version)"
  echo "${release_output}"
  tested_release_version="${release_output##* }"

  just test -p bd3-exec-server --test relay version_skew --test-threads 1
done
