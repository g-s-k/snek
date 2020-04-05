use std::{
    io::{self, Write},
    sync::mpsc,
    thread,
    time::Duration,
};

use termion::{
    color,
    cursor::{Goto, Right},
    event::Key::{self, Char},
    input::TermRead,
    raw::IntoRawMode,
    screen::AlternateScreen,
    style::Reset,
};

mod direction;
mod snake;

use direction::Direction;
use snake::{Pixel, Snake};

/// get terminal size, but half the X value
fn get_adjusted_term_size() -> io::Result<Pixel> {
    termion::terminal_size().map(|(x, y)| Pixel {
        x: (x - 1) / 2,
        y: y - 1,
    })
}

fn generate_game(term_size: Pixel) -> (Snake, Pixel) {
    (
        Snake::new(Pixel::randomize(term_size)),
        Pixel::randomize(term_size),
    )
}

enum UserEvent {
    // meta
    Quit,
    NewGame,
    // movement
    Turn(Direction),
}

fn main() -> io::Result<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for evt in io::stdin().keys() {
            match evt.unwrap() {
                // meta
                Char('q') | Key::Esc => tx.send(UserEvent::Quit),
                Char('n') => tx.send(UserEvent::NewGame),
                // movement
                Char('h') | Char('a') | Key::Left => tx.send(UserEvent::Turn(Direction::Left)),
                Char('j') | Char('s') | Key::Down => tx.send(UserEvent::Turn(Direction::Down)),
                Char('k') | Char('w') | Key::Up => tx.send(UserEvent::Turn(Direction::Up)),
                Char('l') | Char('d') | Key::Right => tx.send(UserEvent::Turn(Direction::Right)),
                _ => Ok(()),
            }
            .unwrap()
        }
    });

    let (mut snek, mut food) = generate_game(get_adjusted_term_size()?);

    let mut screen = AlternateScreen::from(io::stdout().into_raw_mode()?);
    write!(screen, "{}", termion::cursor::Hide)?;

    let lost = loop {
        let current_term_size = get_adjusted_term_size()?;
        let snek_head = snek.head();
        let snek_pixels = snek.rasterize(current_term_size);

        if snek_head.r#in(&snek_pixels) {
            break true;
        }

        if snek_head == food {
            snek.grow();

            while food == snek_head || food.r#in(&snek_pixels) {
                food = Pixel::randomize(current_term_size);
            }
        }

        // clear
        write!(screen, "{}", termion::clear::All)?;
        // food
        write!(
            screen,
            "{}{}  {}",
            Goto(food.x * 2 + 1, food.y + 1),
            color::Bg(color::LightRed),
            Reset,
        )?;
        // snake
        write!(screen, "{}", color::Bg(color::Green))?;
        write!(screen, "{}  ", Goto(snek_head.x * 2 + 1, snek_head.y + 1))?;
        for (y, x_vals) in snek_pixels {
            write!(screen, "{}", Goto(1, y + 1))?;

            let mut last_x = None;
            for x in x_vals {
                let to_move = x - last_x.map(|l| l + 1).unwrap_or_default();

                if to_move != 0 {
                    write!(screen, "{}", Right(to_move * 2))?;
                }

                write!(screen, "  ")?;

                last_x = Some(x);
            }
        }
        write!(screen, "{}", Reset)?;
        screen.flush()?;

        let term_size = get_adjusted_term_size()?;
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(UserEvent::Quit) => break false,
            Ok(UserEvent::Turn(direction)) => {
                snek.turn(direction);
                snek.advance(term_size);
            }
            Ok(UserEvent::NewGame) => {
                let (s, f) = generate_game(term_size);
                snek = s;
                food = f;
            }
            Err(_) => {
                snek.advance(term_size);
            }
        }
    };

    drop(screen);

    if lost {
        println!("Ha ha! Loser.");
    } else {
        println!("bye");
    }

    println!("Score: {}", snek.len());

    Ok(())
}
