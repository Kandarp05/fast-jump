use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::io::RawFd;

/// Temporarily redirects the process stdin/stdout to /dev/tty.
/// Original descriptors are restored on a drop.
pub struct StdioTtyRedirect {
    original_stdin: RawFd,
    original_stdout: RawFd,
}

impl StdioTtyRedirect {
    pub fn new() -> anyhow::Result<Self> {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|err| anyhow::anyhow!("failed to open /dev/tty: {}", err))?;

        let tty_fd = tty.as_raw_fd();

        // Save the current descriptors so we can restore them later.
        let original_stdin = dup_fd(libc::STDIN_FILENO, "stdin")?;
        let original_stdout = dup_fd(libc::STDOUT_FILENO, "stdout")?;

        // Redirect stdin first, then stdout.
        if let Err(e) = dup2_fd(tty_fd, libc::STDIN_FILENO, "stdin") {
            let _ = close_fd(original_stdin);
            let _ = close_fd(original_stdout);
            return Err(e);
        }

        if let Err(e) = dup2_fd(tty_fd, libc::STDOUT_FILENO, "stdout") {
            let _ = dup2_fd(original_stdin, libc::STDIN_FILENO, "stdin");
            let _ = close_fd(original_stdin);
            let _ = close_fd(original_stdout);
            return Err(e);
        }

        Ok(Self {
            original_stdin,
            original_stdout,
        })
    }
}

impl Drop for StdioTtyRedirect {
    fn drop(&mut self) {
        let _ = dup2_fd(self.original_stdin, libc::STDIN_FILENO, "stdin restore");
        let _ = dup2_fd(self.original_stdout, libc::STDOUT_FILENO, "stdout restore");
        let _ = close_fd(self.original_stdin);
        let _ = close_fd(self.original_stdout);
    }
}

fn dup_fd(fd: RawFd, name: &str) -> anyhow::Result<RawFd> {
    // SAFETY: libc::dup only operates on integer file descriptors.
    let new_fd = unsafe { libc::dup(fd) };
    if new_fd < 0 {
        Err(anyhow::anyhow!(
            "failed to duplicate {}: {}",
            name,
            io::Error::last_os_error()
        ))
    } else {
        Ok(new_fd)
    }
}

fn dup2_fd(from: RawFd, to: RawFd, name: &str) -> anyhow::Result<()> {
    // SAFETY: libc::dup2 only operates on integer file descriptors.
    let rc = unsafe { libc::dup2(from, to) };
    if rc < 0 {
        Err(anyhow::anyhow!(
            "failed to redirect {}: {}",
            name,
            io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}

fn close_fd(fd: RawFd) -> anyhow::Result<()> {
    // SAFETY: libc::close only operates on integer file descriptors.
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        Err(anyhow::anyhow!(
            "failed to close fd: {}",
            io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}
