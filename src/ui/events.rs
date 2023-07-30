use anyhow::{anyhow, Result};

use super::playback::play_audio;
use crossterm::event::{self, Event, KeyCode};
use rodio::{Decoder, Sink};
use std::{fs::File, io::BufReader};
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
