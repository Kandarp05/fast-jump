use crate::Args;
use tui_input::Input;

pub struct App {
    pub search_space : Option<String>,
    pub input: Input,
    pub results : Vec<String>,
    pub selected_i : usize,
    pub should_exit: bool,
    pub final_selection : Option<String>,
}

impl App {
    pub fn new(args : Args, list_length: u16) -> Self {
        Self {
            search_space : args.dir,
            input : Input::default(),
            results: Vec::with_capacity(list_length as usize),
            selected_i: 0,
            should_exit: false,
            final_selection : None,
        }
    }

    pub fn move_up(&mut self) {
        if(self.selected_i > 0) {
            self.selected_i -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if(self.selected_i < self.results.len().saturating_sub(1)) {
            self.selected_i += 1;
        }
    }
}