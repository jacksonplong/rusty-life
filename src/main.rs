use core::hash;
use std::{{io::{stdout,Error,Write}, time}, time::{Duration, SystemTime, Instant}};

use crossterm::{cursor::{MoveTo,Show,Hide}, event::{Event::{Key, Resize}, KeyCode, KeyEventKind}, execute, style::{Print, ResetColor}, terminal::{window_size, Clear, WindowSize, self, EnterAlternateScreen, LeaveAlternateScreen}};
use rand::Rng;

extern crate crossterm;
extern crate rand;

const GAMESIZE: usize = 600;
const DENSITY: f64 = 0.07;
const GAMESPEED: u128 = 1200*100; //update delay in microseconds

const DISP_CHAR: char = '0';

pub struct GameState {
    board: [[bool;GAMESIZE];GAMESIZE],
    playing: bool,
    needs_update: bool,
    view_x: usize,
    view_y: usize,
    view_width: usize,
    view_height: usize,
    running: bool,
    update_delay_micros: u128,
    changes_drawn: bool,
    current_command: InputCommand,
}

#[derive(PartialEq)]
pub enum InputCommand {
    Up,
    Down,
    Left,
    Right,
    Pause,
    Place,
    Pass,
    Quit,
    Resize,
}

fn get_view_size() -> Result<(usize, usize), std::io::Error> {
    let ws: WindowSize = crossterm::terminal::window_size()?;
    return Ok((ws.rows as usize, ws.columns as usize));
}

fn initialize_gamestate(view_height: usize, view_width: usize) -> GameState {
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
        needs_update: false,
        playing: true,
        view_x: halfsize,
        view_y: halfsize,
        view_width: view_width,
        view_height: view_height,
        running: true,
        update_delay_micros: GAMESPEED,
        changes_drawn: false,
        current_command: InputCommand::Pass,
    };

    return gs;
}

fn poll_input() -> Result<InputCommand, std::io::Error> {
    if crossterm::event::poll(Duration::new(0, 0))? { //Only poll for a moment
        match crossterm::event::read()? {
            Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => return Ok(InputCommand::Quit),
                        KeyCode::Char(' ') => return Ok(InputCommand::Pause),
                        KeyCode::Char('q') => return Ok(InputCommand::Quit),
                        KeyCode::Up => return Ok(InputCommand::Up),
                        KeyCode::Left => return Ok(InputCommand::Left),
                        KeyCode::Down => return Ok(InputCommand::Down),
                        KeyCode::Right => return Ok(InputCommand::Right),
                        KeyCode::Char('w') => return Ok(InputCommand::Up),
                        KeyCode::Char('a') => return Ok(InputCommand::Left),
                        KeyCode::Char('s') => return Ok(InputCommand::Down),
                        KeyCode::Char('d') => return Ok(InputCommand::Right),
                        _ => ()
                    }
                }
            },
            crossterm::event::Event::Resize(_width,_height) => return Ok(InputCommand::Resize),
            _ => ()
        }
    }
    return Ok(InputCommand::Pass); //no input 
}

fn execute_input(gs: &mut GameState) {
    match gs.current_command {
        InputCommand::Quit => {
            gs.running = false;
        },

        InputCommand::Pause => {
            gs.playing = !gs.playing;
        },
        InputCommand::Up | InputCommand::Left | InputCommand::Down | InputCommand::Right => {
            move_command(gs);
        },
        InputCommand::Resize => {
            gs.changes_drawn = false;
            return; // let sketchy resizing stuff get handled outside this function
        }
        _ => ()
    }
    gs.current_command = InputCommand::Pass;
}

fn move_command(gs: &mut GameState) {
    match gs.current_command {
        InputCommand::Up => {
            if gs.view_y > 0 {
                gs.view_y -= 1;
                gs.changes_drawn = false;
            }
        },
        InputCommand::Left => {
            if gs.view_x > 0 {
                gs.view_x -= 1;
                gs.changes_drawn = false;
            }
        },
        InputCommand::Down => {
            if gs.view_y < GAMESIZE as usize - gs.view_height {
                gs.view_y += 1;
                gs.changes_drawn = false;
            }
        },
        InputCommand::Right => {
            if gs.view_x < GAMESIZE as usize - gs.view_width {
                gs.view_x += 1;
                gs.changes_drawn = false;
                }
        },
        _ => (),
    }
}

fn draw_screen(gs: &GameState) -> Result<(), std::io::Error> {
    //draw view_x -> view_x + width
    //draw view_y -> view_y + height

    let mut outbuff: Vec<char> = vec![];

    if gs.view_width > GAMESIZE || gs.view_height > GAMESIZE {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Window too large"));
    }

    for i in 0..gs.view_height {
        for j in 0..gs.view_width {
            if gs.board[i as usize + gs.view_y][j as usize + gs.view_x] {
                outbuff.append(&mut vec![DISP_CHAR]);
            } else {
                outbuff.append(&mut vec![' ']);
                // outbuff.append(&mut vec![DISP_CHAR]);
            }
        }
    }
    execute!(stdout(), Clear(crossterm::terminal::ClearType::Purge))?;
    execute!(stdout(), MoveTo(0,0))?;
    execute!(stdout(), Print(outbuff.into_iter().collect::<String>()))?;
    return Ok(());
}

// (i-1, j-1), (i-1, j), (i-1, j+1)
// ( i, j-1),  ( i,j),   ( i, j+1)
// (i+1, j-1), (i+1, j), (i+1, j+1)

fn update_gameboard(gs: &mut GameState) {
    let mut next_board = [[false; GAMESIZE]; GAMESIZE];
    for i in 0..GAMESIZE {
        for j in 0..GAMESIZE {

            let mut neighbor_count: u16 = 0;
            //Upper row
            if i > 0 {
                if j > 0 {
                    if gs.board[i-1][j-1] {neighbor_count += 1};
                }
                if gs.board[i-1][j] {neighbor_count += 1};
                if j < GAMESIZE-1 {
                    if gs.board[i-1][j+1] {neighbor_count += 1};
                }
            }
            
            //same row
            if j > 0 {
                if gs.board[i][j-1] {neighbor_count += 1};
            }
            if j < GAMESIZE-1 {
                if gs.board[i][j+1] {neighbor_count += 1};
            }

            //lower row
            if i < GAMESIZE - 1 {
                if j > 0 {
                    if gs.board[i+1][j-1] {neighbor_count += 1};
                }
                if gs.board[i+1][j] {neighbor_count += 1};
                if j < GAMESIZE-1 {
                    if gs.board[i+1][j+1] {neighbor_count += 1};
                }
            }

            if neighbor_count == 3 {
                next_board[i][j] = true;
            } else if neighbor_count == 2 {
                next_board[i][j] = gs.board[i][j]
            } else if neighbor_count < 2 {
                next_board[i][j] = false;
            } else if neighbor_count > 3 {
                next_board[i][j] = false;
            }
        }
    }
    gs.board = next_board;
    gs.changes_drawn = false;
    gs.needs_update = false;
}

struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = stdout();
        let _ = execute!(out, ResetColor, Show, LeaveAlternateScreen);
        let _ = execute!(out, ResetColor);
        let _ = execute!(out, Show);
        let _ = execute!(out, LeaveAlternateScreen);
    }
}

fn main() -> Result<(), std::io::Error> {

    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Hide)?;
    
    let _guard = CleanupGuard;
    
    let (view_height, view_width) = get_view_size()?;
    let mut gamestate = initialize_gamestate(view_height, view_width);
    
    // Game Loop exited with the running boolean, currently in execute_input
    let mut last_update_time = Instant::now();
    loop {
		// Input Reading and Enforcing
        gamestate.current_command = poll_input()?;
        execute_input(&mut gamestate);
    
        if
        last_update_time.elapsed().as_micros() > gamestate.update_delay_micros
        && gamestate.playing
        {
            update_gameboard(&mut gamestate);
            last_update_time = Instant::now();
        }

        // Display Stuff
        if gamestate.changes_drawn == false {
            if gamestate.current_command == InputCommand::Resize {
                let (view_height, view_width) = get_view_size()?;
                gamestate.view_height = view_height;
                gamestate.view_width = view_width;
                gamestate.current_command = InputCommand::Pass;
            }
            match draw_screen(&gamestate) {
                Err(_e) => {
                    gamestate.running = false;
                },
                _ => {
                    gamestate.changes_drawn = true;
                }
            }
        }

        // Loop Killer
        if gamestate.running == false {
            break;
        }
    }

    // This should run before exiting to the terminal.
    return Ok(());
}
