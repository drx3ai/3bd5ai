#![allow(clippy::expect_used)]

use std::process::Command;

use divan::Bencher;

fn main() {
    divan::main();
}

/// Exercises the Bazel-backed end-to-end benchmark path with a cheap,
/// deterministic Bd3 invocation. Richer scenarios can add separate
/// benchmark binaries without making the shared harness depend on them.
#[divan::bench(sample_count = 20, sample_size = 1)]
fn bd3_help(bencher: Bencher) {
    let bd3 = bd3_utils_cargo_bin::cargo_bin("3bd")
        .expect("3bd binary should be available through Bazel runfiles");

    bencher.bench_local(move || {
        let output = Command::new(&bd3)
            .arg("--help")
            .output()
            .expect("3bd --help should run");
        assert!(output.status.success(), "3bd --help should succeed");
    });
}
