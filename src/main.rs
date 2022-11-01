use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::event::{Event, KeyCode};
use space_invaders::frame::{self, Drawable};
use space_invaders::frame::new_frame;
use space_invaders::invaders::Invaders;
use space_invaders::player::Player;
use space_invaders::render;
use std::{io, thread};
use crossterm::{terminal, ExecutableCommand, event};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{Hide, Show};

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: add audios

    // TODO: audio.play("startup");

    // terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        // per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            // TODO: audio.play("shoot");
                        }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break 'gameloop;
                    },
                    _ => {},
                }
            }
        }

        // updates
        player.update(delta);
        if invaders.update(delta) {
            // TODO: audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            // TODO: audio.play("explode");
        }

        // draw & render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // win or lose
        if invaders.all_killed() {
            // TODO: audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            // TODO: audio.play("lose");
            break 'gameloop;
        }
    }
    
    // cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
