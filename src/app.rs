use crossbeam_channel::{Receiver, Sender};
use crossterm::event;
use tui_input::Input;

use crate::cli;
use crate::engine::{EngineCommand, EngineResult};
use crate::tui::Tui;

pub struct App {
    pub input: Input,
    pub results: Vec<String>,
    pub selected_i: usize,
    pub should_exit: bool,
    pub final_selection: Option<String>,
    pub rx_res: Receiver<EngineResult>,
    pub tx_cmd: Sender<EngineCommand>,
}

impl App {
    pub fn new(
        rx_res: Receiver<EngineResult>,
        tx_cmd: Sender<EngineCommand>,
        list_length: u16,
    ) -> Self {
        Self {
            input: Input::default(),
            results: Vec::with_capacity(list_length as usize),
            selected_i: 0,
            should_exit: false,
            final_selection: None,
            rx_res,
            tx_cmd,
        }
    }

    // Move up the result list
    pub fn move_up(&mut self) {
        if self.selected_i > 0 {
            self.selected_i -= 1;
        }
    }

    // Move down the result list
    pub fn move_down(&mut self) {
        if self.selected_i < self.results.len().saturating_sub(1) {
            self.selected_i += 1;
        }
    }

    pub fn run_event_loop(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        loop {
            tui.terminal.draw(|f| cli::render::draw(f, self))?;

            let mut received_new_res = false;
            while let Ok(new_res) = self.rx_res.try_recv() {
                match new_res {
                    EngineResult::Update(res) => {
                        self.results = res;
                    }
                    EngineResult::Done => {}
                }

                self.clamp_selection();
                received_new_res = true;
            }

            if received_new_res {
                continue;
            }

            if event::poll(std::time::Duration::from_millis(50))? {
                cli::events::handle_events(self)?;
            }

            if self.should_exit {
                break;
            }
        }

        Ok(())
    }

    fn clamp_selection(&mut self) {
        if self.selected_i >= self.results.len() {
            self.selected_i = self.results.len().saturating_sub(1);
        }
    }
}
