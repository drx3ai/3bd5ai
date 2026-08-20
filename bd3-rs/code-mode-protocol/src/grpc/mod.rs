#[cfg(bd3_bazel)]
pub use code_mode_proto::bd3::code_mode::v1::*;

#[cfg(not(bd3_bazel))]
tonic::include_proto!("3bd.code_mode.v1");

pub const MAX_IDENTIFIER_BYTES: usize = 256;
