use std::{
    io::{BufRead, BufReader},
    process::{Child, ChildStdout, ChildStderr, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use std::sync::mpsc::{channel, Receiver};
use std::thread::{self, JoinHandle};

pub struct CommandOutput {
    pub status: Option<ExitStatus>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub timed_out: bool,
}

pub fn run_command_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> std::io::Result<CommandOutput> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (out_lines, err_lines) = stream_outputs(stdout, stderr);

    let start = Instant::now();
    let status = loop {
        if start.elapsed() > timeout {
            child.kill().ok();
            break None;
        }
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        thread::sleep(Duration::from_millis(50));
    };

    let stdout = out_lines.join().unwrap_or_default();
    let stderr = err_lines.join().unwrap_or_default();

    Ok(CommandOutput {
        status,
        stdout,
        stderr,
        timed_out: status.is_none(),
    })
}

fn stream_outputs(
    stdout: ChildStdout,
    stderr: ChildStderr,
) -> (JoinHandle<Vec<String>>, JoinHandle<Vec<String>>) {
    let out_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            println!("[stdout] {}", line);
            lines.push(line);
        }
        lines
    });

    let err_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            eprintln!("[stderr] {}", line);
            lines.push(line);
        }
        lines
    });

    (out_handle, err_handle)
}

