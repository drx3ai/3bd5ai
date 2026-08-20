# Bd3 CLI Runtime for Python SDK

Platform-specific runtime package consumed by the published `bd3`.

This package is staged during release so the SDK can pin an exact Bd3 CLI
version without checking platform binaries into the repo.

`bd3-cli-bin` is intentionally wheel-only. Do not build or publish an
sdist for this package.
