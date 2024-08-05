use crossterm::{
    cursor::{self},
    event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal, QueueableCommand,
};
use std::{
    io::{self, stdout, Write},
    thread,
    time::Duration,
};

fn run_terminal() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut run = true;

    let sep_char = "─";
    let mut stdout = stdout();
    let (mut width, mut height) = terminal::size()?;
    let mut sep_line = sep_char.repeat(width as usize);
    let clear_terminal_cmd = terminal::Clear(terminal::ClearType::All);

    let mut chat_prompt = String::new();
    // CTRL+C terminal kill
    let terminal_kill_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

    let _enter_prompt = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

    while run {
        while event::poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(key_event) => {
                    if key_event == terminal_kill_event {
                        run = false;
                    }
                    if let KeyCode::Char(ch) = key_event.code {
                        chat_prompt.push(ch);
                    }
                }

                Event::Resize(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    sep_line = sep_char.repeat(width as usize);
                }

                _ => {}
            }
        }

        stdout.queue(clear_terminal_cmd)?;

        // draw seperator
        stdout.queue(cursor::MoveTo(0, height - 2))?;
        stdout.write(sep_line.as_bytes())?;

        // draw prompt
        stdout.queue(cursor::MoveTo(0, height - 1))?;
        stdout.write(chat_prompt.as_bytes())?;

        stdout.flush()?;
        thread::sleep(Duration::from_millis(33));
    }

    Ok(())
}
