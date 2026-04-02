use crate::tty_redirect::StdioTtyRedirect;
use anyhow::Context;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::io::Stdout;

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    _stdio_redirect: StdioTtyRedirect,
}

impl Tui {
    pub fn init(max_list_length: u16) -> anyhow::Result<Self> {
        // Inline viewport performs cursor queries against process stdio.
        // When stdout is captured by a shell wrapper, we temporarily redirect
        // stdin/stdout to /dev/tty so inline rendering still works.
        let stdio_redirect = StdioTtyRedirect::new()?;

        enable_raw_mode().context("failed to enable raw mode")?;

        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = match Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(1 + max_list_length),
            },
        ) {
            Ok(terminal) => terminal,
            Err(err) => {
                let _ = disable_raw_mode();
                return Err(err.into());
            }
        };

        if let Err(err) = execute!(terminal.backend_mut(), Hide) {
            let _ = disable_raw_mode();
            return Err(err.into());
        };

        Ok(Self {
            terminal,
            _stdio_redirect: stdio_redirect,
        })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show);
    }
}
