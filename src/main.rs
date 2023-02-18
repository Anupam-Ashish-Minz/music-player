use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rodio::{Decoder, OutputStream, PlayError, Sink, StreamError};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    thread,
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

#[derive(Debug)]
enum AudioError {
    IOError(std::io::Error),
    PlayError(PlayError),
    StreamError(StreamError),
}

impl std::error::Error for AudioError {}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::IOError(err) => write!(f, "{}", err),
            AudioError::PlayError(err) => write!(f, "{}", err),
            AudioError::StreamError(err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for AudioError {
    fn from(err: std::io::Error) -> Self {
        AudioError::IOError(err)
    }
}

impl From<PlayError> for AudioError {
    fn from(err: PlayError) -> Self {
        AudioError::PlayError(err)
    }
}

impl From<StreamError> for AudioError {
    fn from(err: StreamError) -> Self {
        AudioError::StreamError(err)
    }
}

impl From<rodio::decoder::DecoderError> for AudioError {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        AudioError::PlayError(PlayError::DecoderError(err))
    }
}

fn play_audio(sink: &Sink, file_name: &str) -> Result<(), AudioError> {
    let file = BufReader::new(File::open(file_name)?);
    let source = Decoder::new(file)?;
    sink.append(source);

    return Ok(());
}

fn draw_lists(list: Vec<String>) -> Result<(), AudioError> {
    let list_items: Vec<_> = list.iter().map(|x| ListItem::new(&x[..])).collect();
    let mut list_state = ListState::default();
    let mut selection_i: Option<usize> = None;

    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (_stream, stream_handler) = OutputStream::try_default()?;

    let sink = Sink::try_new(&stream_handler)?;
    sink.set_volume(0.25);

    loop {
        thread::sleep(Duration::from_millis(15));
        terminal.draw(|f| {
            let size = f.size();

            let list = List::new(list_items.clone())
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        if crossterm::event::poll(Duration::from_millis(300))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('j') => {
                        // move to next item in list
                        if let Some(i) = selection_i {
                            if i < list_items.len() - 1 {
                                selection_i = Some(i + 1);
                            }
                        } else {
                            selection_i = Some(0);
                        }
                        list_state.select(selection_i);
                    }
                    KeyCode::Char('k') => {
                        // move to prev item in list
                        if let Some(i) = selection_i {
                            if i > 0 {
                                selection_i = Some(i - 1);
                            }
                        }
                        list_state.select(selection_i);
                    }
                    KeyCode::Char('g') => {
                        // move to top of the list
                        selection_i = Some(0);
                        list_state.select(selection_i);
                    }
                    KeyCode::Char('G') => {
                        // move to bottom of the list
                        selection_i = Some(list_items.len() - 1);
                        list_state.select(selection_i);
                    }
                    KeyCode::Esc | KeyCode::Char('x') => {
                        // unselect
                        selection_i = None;
                        list_state.select(selection_i);
                    }
                    KeyCode::Enter => {
                        if let Some(song_index) = selection_i {
                            play_audio(&sink, &list[song_index])?;
                        }
                    }
                    _ => {}
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

fn get_file_list(root: PathBuf, list: &mut Vec<String>) -> Result<(), std::io::Error> {
    let mut rval = Ok(());
    for file in fs::read_dir(root)? {
        let file = file?;
        let file_type = file.file_type()?;
        if file_type.is_file() {
            list.push(file.path().to_string_lossy().to_string());
        } else if file_type.is_dir() {
            rval = get_file_list(file.path(), list);
        }
    }
    return rval;
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[arg(short, long, default_value_t = String::from("assets"))]
    path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file_list = Vec::new();
    get_file_list(PathBuf::from(&args.path[..]), &mut file_list)?;

    draw_lists(file_list)?;

    return Ok(());
}
