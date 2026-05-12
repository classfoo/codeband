use crate::tools::driver::ChatSubprocessSpec;
use anyhow::Context;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;

/// Runs the chat subprocess with piped stdout; sends each read chunk to `delta_tx` as it arrives.
/// Returns merged stdout+stderr output and process exit code.
pub async fn stream_chat_subprocess(
    spec: &ChatSubprocessSpec,
    cwd: &Path,
    delta_tx: tokio::sync::mpsc::Sender<String>,
) -> anyhow::Result<(String, i32)> {
    let mut cmd = Command::new(&spec.program);
    for a in &spec.args {
        cmd.arg(a);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }
    cmd.current_dir(cwd);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().context("spawn chat subprocess")?;

    let stdout = child.stdout.take().context("stdout pipe")?;
    let stderr = child.stderr.take().context("stderr pipe")?;

    let mut reader = BufReader::new(stdout);
    let mut stdout_acc = String::new();
    let mut buf = vec![0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await.context("read stdout")?;
        if n == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
        stdout_acc.push_str(&chunk);
        if delta_tx.send(chunk).await.is_err() {
            break;
        }
    }

    let mut stderr_reader = BufReader::new(stderr);
    let mut stderr_acc = String::new();
    stderr_reader
        .read_to_string(&mut stderr_acc)
        .await
        .context("read stderr")?;

    let status = child.wait().await.context("wait subprocess")?;
    let code = status.code().unwrap_or(1);
    let merged = crate::tools::driver::merge_shell_output(&stdout_acc, &stderr_acc);
    Ok((merged, code))
}
