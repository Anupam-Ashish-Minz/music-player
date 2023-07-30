use super::events::handle_event;
use super::playback::AudioError;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rodio::{OutputStream, Sink};
use std::{thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

pub fn draw_lists(list: Vec<String>) -> Result<(), AudioError> {
    let list_items: Vec<_> = list.iter().map(|x| ListItem::new(&x[..])).collect();
    let mut list_state = ListState::default();

    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (_stream, stream_handler) = OutputStream::try_default()?;

    let volume = 1.0;

    let sink = Sink::try_new(&stream_handler)?;
    sink.set_volume(volume);

    let list_len = list.len();

    loop {
        thread::sleep(Duration::from_millis(15));
        terminal.draw(|f| {
            let size = f.size();

            let list = List::new(list_items.clone())
                .block(Block::default().title("Music List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        if crossterm::event::poll(Duration::from_millis(300))? {
            match handle_event(&mut list_state, list_len, &list, &sink) {
                Ok(_) => {}
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    terminal.show_cursor()?;
    Ok(())
}
