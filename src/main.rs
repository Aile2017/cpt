use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "cpt: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    if std::env::args_os().len() != 1 {
        return Err("command-line arguments are not supported".into());
    }
    forward(
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
        set_clipboard,
    )
}

fn forward(
    input: &mut impl Read,
    output: &mut impl Write,
    clipboard: impl FnOnce(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    let mut bytes = Vec::new();
    input
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read standard input: {error}"))?;
    output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .map_err(|error| format!("cannot write standard output: {error}"))?;
    clipboard(&bytes)
}

fn set_clipboard(bytes: &[u8]) -> Result<(), String> {
    let system_root =
        std::env::var_os("SystemRoot").ok_or("SystemRoot is not set; cannot locate clip.exe")?;
    let path = PathBuf::from(system_root).join("System32").join("clip.exe");
    let mut child = Command::new(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("cannot start {}: {error}", path.display()))?;

    let mut stdin = child.stdin.take().expect("piped child standard input");
    let write_result = stdin.write_all(bytes);
    drop(stdin);
    // Close stdin to signal EOF, and reap the child even if writing failed.
    let wait_result = child.wait();
    write_result.map_err(|error| format!("cannot write to clip.exe: {error}"))?;
    let status = wait_result.map_err(|error| format!("cannot wait for clip.exe: {error}"))?;
    if !status.success() {
        return Err(format!("clip.exe failed: {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::forward;
    use std::io::{self, Write};

    #[test]
    fn duplicates_bytes_without_interpreting_encoding() {
        for input in [
            b"".as_slice(),
            b"hello\r\n",
            b"\xef\xbb\xbfUTF-8",
            b"\x82\xa0\x82\xa2\r\n",
            b"\xff\0\xfe",
        ] {
            let mut output = Vec::new();
            let mut clipboard = Vec::new();
            forward(&mut &input[..], &mut output, |bytes| {
                clipboard.extend_from_slice(bytes);
                Ok(())
            })
            .unwrap();
            assert_eq!(output, input);
            assert_eq!(clipboard, input);
        }
    }

    struct FailingOutput;

    impl Write for FailingOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
    }

    #[test]
    fn output_failure_stops_before_clipboard() {
        let error = forward(&mut &b"text"[..], &mut FailingOutput, |_| {
            panic!("clipboard must not be called")
        })
        .unwrap_err();
        assert!(error.contains("cannot write standard output"));
    }

    #[test]
    fn clipboard_failure_is_reported_after_output() {
        let mut output = Vec::new();
        let error = forward(&mut &b"text"[..], &mut output, |_| {
            Err("clip.exe failed".into())
        })
        .unwrap_err();
        assert_eq!(error, "clip.exe failed");
        assert_eq!(output, b"text");
    }
}
