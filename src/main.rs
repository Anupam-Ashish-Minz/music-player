use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rodio::{Decoder, OutputStream, OutputStreamHandle, PlayError, Sink, StreamError};
use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
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
    UnknownError,
    IOError(std::io::Error),
    PlayError(PlayError),
    StreamError(StreamError),
}

impl std::error::Error for AudioError {}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::UnknownError => write!(f, "unknown error"),
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

fn play_audio(sink: &Sink, source: Decoder<BufReader<File>>) {
    sink.stop();
    sink.append(source);
    sink.play();
}

fn handle_event(
    list_state: &mut ListState,
    list_len: usize,
    list: &Vec<String>,
    sink: &Sink,
) -> Result<()> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => return Err(anyhow!("quit")),
            KeyCode::Char('j') => {
                // move to next item in list
                if let Some(i) = list_state.selected() {
                    if i + 1 < list_len {
                        list_state.select(Some(i + 1));
                    }
                } else {
                    list_state.select(Some(0));
                }
            }
            KeyCode::Char('k') => {
                if let Some(i) = list_state.selected() {
                    if i > 0 {
                        list_state.select(Some(i - 1));
                    }
                }
            }
            KeyCode::Char('g') => {
                // move to top of the list
                list_state.select(Some(0));
            }
            KeyCode::Char('G') => {
                // move to bottom of the list
                list_state.select(Some(list_len - 1));
            }
            KeyCode::Char('h') => {
                // go back 10s
            }
            KeyCode::Char('l') => {
                // move forawrd 10s
            }
            KeyCode::Char('c') => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            KeyCode::Char('x') => {
                sink.stop();
            }
            KeyCode::Esc => {
                // unselect
                list_state.select(None);
            }
            KeyCode::Enter => {
                if let Some(i) = list_state.selected() {
                    let file_name = &list[i];
                    let file = BufReader::new(File::open(file_name)?);
                    let source = Decoder::new(file)?;
                    play_audio(&sink, source);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_lists(list: Vec<String>) -> Result<(), AudioError> {
    let list_items: Vec<_> = list.iter().map(|x| ListItem::new(&x[..])).collect();
    let mut list_state = ListState::default();

    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (_stream, stream_handler) = OutputStream::try_default()?;

    let volume = 1.0;

    // this is just a placeholder the sink is overridden when the song is to
    // be played that is during the Enter event
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
