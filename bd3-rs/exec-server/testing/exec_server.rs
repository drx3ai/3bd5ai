//! Minimal exec-server fixture for Bazel-only integration tests.
//!
//! Linking only exec-server avoids depending on the full Bd3 CLI binary
//! when a test only needs a WebSocket executor endpoint. It handles the arg0
//! helper mode because sandboxed process requests re-exec this binary.

use bd3_exec_server::ExecServerRuntimePaths;
use bd3_http_client::HttpClientFactory;
use bd3_http_client::OutboundProxyPolicy;
#[cfg(unix)]
use std::ffi::OsStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(unix)]
    {
        let mut args = std::env::args_os();
        let _ = args.next();
        if args.next().as_deref()
            == Some(OsStr::new(
                bd3_exec_server::BD3_ARG0_EXEC_HELPER_ARG1,
            ))
        {
            bd3_exec_server::run_arg0_exec_helper_main();
        }
    }

    let current_exe = std::env::current_exe()?;
    let runtime_paths =
        ExecServerRuntimePaths::new(current_exe, /*bd3_linux_sandbox_exe*/ None)?;
    bd3_exec_server::run_main(
        "ws://127.0.0.1:0",
        runtime_paths,
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .await
}
