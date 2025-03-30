use std::{io::stdout, time::Duration};

use crossterm::{event::{Event::Key, KeyCode, KeyEventKind}, execute, style::Print, terminal::{Clear, WindowSize}};
use rand::Rng;

extern crate crossterm;
extern crate rand;

const GAMESIZE: usize = 1000;
const DENSITY: f64 = 0.3;
const GAMESPEED: i32 = 1;

const DISP_CHAR: char = '0';

pub struct GameState {
    board: [[bool;GAMESIZE];GAMESIZE],
    view_x: usize,
    view_y: usize,
    running: bool,
    speed: i32
}

pub enum InputCommand {
    Up,
    Down,
    Left,
    Right,
    Pause,
    Place,
    Pass,
    Quit
}

fn initialize_gamestate() -> GameState {
    let mut rng = rand::rng();
    let mut brd = [[false; GAMESIZE]; GAMESIZE];

    for i in 0..GAMESIZE {
        for j in 0..GAMESIZE {
            brd[i][j] = rng.random_bool(DENSITY);
        }
    }

    let halfsize = GAMESIZE/2;

    let gs = GameState {
        board: brd,
        view_x: halfsize,
        view_y: halfsize,
        running: true,
        speed: GAMESPEED
    };

    return gs;
}

fn poll_input() -> Result<InputCommand, std::io::Error> {
    if crossterm::event::poll(Duration::new(0, 0))? {
        match crossterm::event::read()? {
            Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => return Ok(InputCommand::Quit),
                        _ => ()
                    }
                }
            }
            _ => ()
        }
    }
    return Ok(InputCommand::Pass);
}

fn execute_input(gs: &mut GameState, ic: InputCommand) {
    match ic {
        InputCommand::Quit => {
            gs.running = false;
            let _ = crossterm::terminal::disable_raw_mode();
        },
        _ => ()
    }
}

fn draw_screen(gs: &GameState) -> Result<(), std::io::Error> {
    let ws: WindowSize = crossterm::terminal::window_size()?;
    //draw view_x -> view_x + width
    //draw view_y -> view_y + height

    let mut outbuff: Vec<char> = vec![];

    for i in 0..ws.rows {
        for j in 0..ws.columns {
            if gs.board[i as usize][j as usize] {
                outbuff.append(&mut vec![DISP_CHAR]);
            } else {
                outbuff.append(&mut vec![' ']);
            }
        }
    }
    execute!(stdout(), Clear(crossterm::terminal::ClearType::Purge))?;
    execute!(stdout(), Print(outbuff.into_iter().collect::<String>()))?;
    return Ok(());
}

fn main() {
    
    let _ = crossterm::terminal::enable_raw_mode();

    let mut gamestate = initialize_gamestate();

    loop {
        let _ = update(&mut gamestate);

        if gamestate.running == false {
            break;
        }
    }
}

fn update(gs: &mut GameState) -> Result<(), std::io::Error>{
    *gs = initialize_gamestate();
    // poll input
    let input = poll_input()?;
    execute_input(gs, input);
    // update gamestate
    // draw screen
    let _ = draw_screen(gs);

    return Ok(());
}
