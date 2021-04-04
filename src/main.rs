extern crate termion;
extern crate indoc;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::cursor::DetectCursorPos;
use std::io::Write;

const MANUAL: &str = indoc::indoc! {r#"
                      .   '||      '||
              ....  .||.   || ...   || ...
             ||. '   ||    ||'  ||  ||'  ||
             . '|..  ||    ||    |  ||    |
             |'..|'  '|.'  '|...'   '|...'


    Welcome to stbb, the Simple Terminal Blackboard.

    You can type anywhere on the screen but you can't
    scroll or save your work. This program doesn't
    even store your text in memory. It only sends the
    appropriate ANSI escape codes to the terminal to
    temporarily display your text.

    There are two modes of operation: Normal mode and
    insert mode.

    ============= Normal mode commands ==============
    q:                 Quit the program
    h j k l:           Move the cursor
    b e:               Jump back/forward by 10 spaces
    c:                 Clear the entire screen
    i:                 Enter insert mode

    ============= Insert mode commands ==============
    Ctrl-[ or ESC:     Go back to normal mode
    other keys:        type stuff
"#};

enum Mode {
    Normal,
    Insert {
        entrance: (u16, u16),
    },
}

struct App {
    raw_terminal: termion::raw::RawTerminal<std::io::Stdout>,
    mode: Mode,
}

impl App {
    fn new() -> Result<App, std::io::Error> {
        Ok(App {
            mode: Mode::Normal,
            raw_terminal: std::io::stdout().into_raw_mode()?,
        })
    }

    fn show_manual(&mut self) -> Result<(), std::io::Error> {
        // Decide where to print the manual
        let terminal_size = termion::terminal_size()?;
        let manual_size = (
            MANUAL.lines().into_iter().fold(0, |longest, line| std::cmp::max(longest, line.len())) as u16,
            MANUAL.lines().count() as u16,
        );
        let manual_origin = (
            (terminal_size.0 - manual_size.0) / 2,  // TODO overflow on small screen
            (terminal_size.1 - manual_size.1) / 2);

        // Decide where to leave the cursor afterwards
        let cursor_relative: (u16, u16) = (11, 16);
        let cursor_absolute = (
            manual_origin.0 + cursor_relative.0,
            manual_origin.1 + cursor_relative.1);

        for (index, line) in MANUAL.lines().enumerate() {
            write!(self.raw_terminal, "{}{}",
                   termion::cursor::Goto(manual_origin.0, manual_origin.1 + (index as u16)),
                   line)?;
        }
        write!(self.raw_terminal, "{}", termion::cursor::Goto(cursor_absolute.0, cursor_absolute.1))?;
        self.raw_terminal.flush()?;

        Ok(())
    }

    fn handle_input(&mut self, key: Key) -> Result<bool, std::io::Error> {
        match self.mode {
            Mode::Normal => match key {
                Key::Char('q') => return Ok(false),
                Key::Char('b') => write!(self.raw_terminal, "{}", termion::cursor::Left(10))?,
                Key::Char('e') => write!(self.raw_terminal, "{}", termion::cursor::Right(10))?,
                Key::Char('h') => write!(self.raw_terminal, "{}", termion::cursor::Left(1))?,
                Key::Char('l') => write!(self.raw_terminal, "{}", termion::cursor::Right(1))?,
                Key::Char('k') => write!(self.raw_terminal, "{}", termion::cursor::Up(1))?,
                Key::Char('j') => write!(self.raw_terminal, "{}", termion::cursor::Down(1))?,
                Key::Char('c') => write!(self.raw_terminal, "{}", termion::clear::All)?,
                Key::Char('i') => {
                    self.mode = Mode::Insert{
                        entrance: self.raw_terminal.cursor_pos()?,
                    };
                    write!(self.raw_terminal, "{}", termion::cursor::SteadyBar)?;
                }
                _ => {}
            },
            Mode::Insert { entrance } => match key {
                Key::Char('\n') => {
                    let cursor_pos = self.raw_terminal.cursor_pos()?;
                    write!(self.raw_terminal, "{}", termion::cursor::Goto(entrance.0, cursor_pos.1 + 1))?;
                }
                Key::Char(c) => write!(self.raw_terminal, "{}", c)?,
                Key::Ctrl('[') | Key::Esc => {
                    self.mode = Mode::Normal;
                    write!(self.raw_terminal, "{}", termion::cursor::SteadyBlock)?;
                }
                Key::Backspace => write!(self.raw_terminal, "{} {}",
                                         termion::cursor::Left(1),
                                         termion::cursor::Left(1))?,
                _ => {}
            }
        };
        self.raw_terminal.flush()?;
        Ok(true)
    }

    fn clear_screen(&mut self) -> Result<(), std::io::Error> {
        write!(self.raw_terminal, "{}{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1))?;
        self.raw_terminal.flush()?;
        Ok(())
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        self.clear_screen()?;
        self.show_manual()?;
        for key in std::io::stdin().keys() {
            if !self.handle_input(key?)? {
                break
            }
        }
        self.clear_screen()?;
        Ok(())
    }
}

// TODO:
// - Better error reporting
// - Revert terminal state even on error
// - Don't clear the screen on exit
//   - Look into termion alternative screen but
//     make sure not to capture panic output
// - Implement Ctrl-z for minimizing the GUI
// - Deal with tiny screen size
//  - Show compact manual
// - Add block visual mode
fn main() -> Result<(), std::io::Error> {
    Ok(App::new()?.run()?)
}
