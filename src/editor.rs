use std::io::{Write, stdout};

use anyhow::Ok;
use crossterm::{ExecutableCommand, QueueableCommand, cursor, event::{self, read}, style::{self, Color, Stylize, style}, terminal};

enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    AddChar(char),
    NewLine,

    EnterMode(Mode),
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    stdout: std::io::Stdout,
    size: (u16, u16),
    cx: u16,
    cy: u16,
    mode: Mode,
}

impl Drop for Editor {
    fn drop(&mut self) {
        _ = self.stdout.flush();
        _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    pub fn new() -> anyhow::Result<Self> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(Editor {
            stdout,
            size: terminal::size()?,
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
        })
    }

    pub fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_statuline()?;
        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn draw_statuline(&mut self) -> anyhow::Result<()> {
        let (bg, fg) = ("#fff", "#000");
        let mode = format!(" {:?} ", self.mode).to_uppercase();
        let file = " src/main.rs ";
        let pos = format!(" {}:{} ", self.cy, self.cx);

        let file_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - 2;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 -2))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(Color::Black).on(Color::White).bold(),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            "".with(Color::White).on(Color::Grey)
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize).with(Color::Black).on(Color::Grey)
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            "".with(Color::White).on(Color::Grey)
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            pos.with(Color::Black).on(Color::White).bold(),
        ))?;

        Ok(())
    }


    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.draw()?;
            if let Some(action) = self.handle_event(read()?)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.cy = self.cy.saturating_sub(1)
                    },
                    Action::MoveDown => {
                        self.cy += 1u16
                    },
                    Action::MoveLeft => {
                        self.cx = self.cx.saturating_sub(1)
                    },
                    Action::MoveRight => {
                        self.cx += 1u16
                    },
                    Action::EnterMode(new_mode) => {
                        self.mode = new_mode;
                    },
                    Action::AddChar(c) => {
                        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
                        self.stdout.queue(style::Print(c))?;
                        self.cx += 1;
                    },
                    Action::NewLine => {
                        self.cx = 0;
                        self.cy += 1;
                    }
                }
            }
        }


        Ok(())
    }

    fn handle_event(&mut self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        if matches!(ev, event::Event::Resize(_, _)) {
            self.size = terminal::size()?;
        }

        match self.mode {
            Mode::Normal => self.handle_normal_event(ev),
            Mode::Insert => self.handle_insert_event(ev),
        }
    }

    fn handle_normal_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                event::KeyCode::Up | event::KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                event::KeyCode::Down | event::KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                event::KeyCode::Left | event::KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Right | event::KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
                event::KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                _ => Ok(None),
            }
            _ => Ok(None),
        };
        Ok(action)?
    }

    fn handle_insert_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                event::KeyCode::Char(c) => Ok(Some(Action::AddChar(c))),
                event::KeyCode::Enter => Ok(Some(Action::NewLine)),
                _ => Ok(None),
            },
            _ => Ok(None),
        };
        Ok(action)?
    }
}
