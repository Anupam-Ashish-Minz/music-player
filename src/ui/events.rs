use anyhow::{anyhow, Result};

use crossterm::event::{self, Event, KeyCode};
use rodio::{Decoder, Sink, Source};
use std::{fs::File, io::BufReader, time::Duration};
use tui::widgets::ListState;

pub fn handle_event(
    list_state: &mut ListState,
    list_len: usize,
    list: &Vec<String>,
    sink: &Sink,
) -> Result<()> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => return Err(anyhow!("regular exit")),
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
                // move forward 10s
                if !sink.empty() {
                    if let Some(i) = list_state.selected() {
                        let open_file = &list[i];
                        let file = BufReader::new(File::open(open_file)?);
                        let source = Decoder::new(file)?;
                        let source = source.skip_duration(Duration::new(14, 0));
                        sink.stop();
                        sink.append(source);
                        sink.play();
                    }
                }
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
                    let open_file = &list[i];
                    let file = BufReader::new(File::open(open_file)?);
                    let source = Decoder::new(file)?;
                    sink.stop();
                    sink.append(source);
                    sink.play();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
