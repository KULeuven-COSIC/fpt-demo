use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
  disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::error::Error;
use std::io;

use std::time::Instant;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::Rect;
use tui::widgets::{Block, Borders};
use tui::{Frame, Terminal};

use homomorphic_game_of_life::gol::*;

#[cfg(feature = "fpga")]
use tfhe::boolean::server_key::FpgaGates;

use std::env;
use tfhe::boolean::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut current_dir = env::current_dir().unwrap();
  current_dir.push("demos/game-of-life/initial_state");
  let csv_path = current_dir.to_str().unwrap();

  // read the dimensions and initial state
  let (dimensions, initial_state): ((usize, usize), Vec<bool>) = read_csv(csv_path).unwrap();

  // generate the client key
  let client_key = ClientKey::new(&DEMO_PARAMETERS);

  // generate the server key
  let server_key = ServerKey::new(&client_key);

  #[cfg(feature = "fpga")]
  server_key.enable_fpga();

  // encrypt false three times
  let mut zeros = (
    Vec::<Ciphertext>::new(),
    Vec::<Ciphertext>::new(),
    Vec::<Ciphertext>::new(),
  );
  for _ in 0..FPGA_BOOTSTRAP_PACKING {
    zeros.0.push(client_key.encrypt(false));
    zeros.1.push(client_key.encrypt(false));
    zeros.2.push(client_key.encrypt(false));
  }

  // encrypt the initial configuration
  let initial_state: Vec<Ciphertext> = initial_state
    .into_iter()
    .map(|x| client_key.encrypt(x))
    .collect();

  // build the board
  let board = Board::new(dimensions.1, initial_state, client_key);

  // create app and run it
  let res = run_app(&mut terminal, board, &server_key, &zeros);

  // restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{:?}", err)
  }

  Ok(())
}

fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  mut board: Board,
  server_key: &ServerKey,
  zeros: &(Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
) -> io::Result<()> {
  loop {
    terminal.draw(|f| ui(f, board.clone()))?;
    let now = Instant::now();
    board.update(server_key, zeros);
    board.tick_time = now.elapsed();
  }
}

fn ui<B: Backend>(f: &mut Frame<B>, board: Board) {
  // Surrounding block
  let size = f.size();
  let block = Block::default()
    .borders(Borders::ALL)
    .title("Conway's Game of Life");
  f.render_widget(block, size);

  // Game block
  let block = Block::default().borders(Borders::ALL);
  f.render_widget(
    block,
    Rect::new(
      1,
      1,
      (board.dimensions.0 + 2) as u16,
      (board.dimensions.1 + 2) as u16,
    ),
  );

  // Game board
  f.render_widget(board, f.size());
}
