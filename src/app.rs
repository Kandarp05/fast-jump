use crate::Args;
use crate::engine::EngineCommand;
use crossbeam_channel::{Receiver, Sender};
use tui_input::Input;

pub struct App {
    pub input: Input,
    pub results: Vec<String>,
    pub selected_i: usize,
    pub should_exit: bool,
    pub final_selection: Option<String>,
    pub rx_res: Receiver<Vec<String>>,
    pub tx_cmd: Sender<EngineCommand>,
}

impl App {
    pub fn new(
        rx_res: Receiver<Vec<String>>,
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
}
