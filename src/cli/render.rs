use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    // Max no. of results to show.
    let result_list_size = app.results.len().saturating_sub(1);

    // Split in 2, 0 -> Input, 1 -> Results list
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(result_list_size as u16),
        ])
        .split(f.area());

    // Split in 2, 0 -> Static prompt, 1 -> actual input
    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(6), Constraint::Min(1)])
        .split(main_chunks[0]);

    // Static prompt
    let prompt = Paragraph::new(Span::raw(" fj > ").fg(Color::Cyan).bold());
    f.render_widget(prompt, input_chunks[0]);

    let width = input_chunks[1].width.max(1) as usize;
    let scroll = app.input.visual_scroll(width);

    // The actual input
    let input_text = Paragraph::new(app.input.value()).scroll((0, scroll as u16));
    f.render_widget(input_text, input_chunks[1]);

    let cursor_offset = app.input.visual_cursor().saturating_sub(scroll) as u16;
    f.set_cursor_position((input_chunks[1].x + cursor_offset, input_chunks[1].y));

    // Result list
    let mut result_lines = Vec::new();
    for (i, res) in app.results.iter().take(result_list_size).enumerate() {
        if i == app.selected_i {
            result_lines.push(Line::from(vec![
                Span::styled("  > ", Style::default().fg(Color::Yellow)),
                Span::styled(res.as_str(), Style::default().fg(Color::Yellow).bold()),
            ]));
        } else {
            result_lines.push(Line::from(vec![Span::raw("    "), Span::raw(res.as_str())]));
        }
    }
    f.render_widget(Paragraph::new(result_lines), main_chunks[1]);
}
