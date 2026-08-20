#!/bin/bash

# Set "chatgpt.cliExecutable": "/Users/<USERNAME>/code/3bd/scripts/debug-3bd.sh" in VSCode settings to always get the 
# latest bd3-rs binary when debugging Bd3 Extension.


set -euo pipefail

BD3_RS_DIR=$(realpath "$(dirname "$0")/../bd3-rs")
(cd "$BD3_RS_DIR" && cargo run --quiet --bin bd3 -- "$@")