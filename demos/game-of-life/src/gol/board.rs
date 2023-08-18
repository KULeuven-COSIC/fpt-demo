use std::time::Duration;

use tfhe::boolean::prelude::*;

use super::operators::is_alive;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};

use tui::widgets::Widget;

/// a Game of Life board structure
///
/// # Fields
///
/// * `dimensions`: the dimensions of the board
/// * `states`: encrypted states of the cells
#[derive(Clone)]
pub struct Board {
  pub dimensions: (usize, usize),
  pub states: Vec<Ciphertext>,
  pub client_key: ClientKey,
  pub tick_time: Duration,
}

impl Board {
  /// create a new board
  ///
  /// # Arguments
  ///
  /// * `n_cols`: number of columns
  /// * `states`: encrypted states of the cells in the initial configuration
  ///
  /// # Example
  ///
  /// ```
  /// use homomorphic_game_of_life::*;
  ///
  /// // numbers of rows and columns in the board
  /// let (n_rows, n_cols): (usize, usize) = (6,6);
  ///
  /// // generate the client key
  /// let (client_key, _) = gen_keys();
  ///
  /// // initial configuration
  /// let states = vec![true, false, false, false, false, false,
  ///                   false, true, true, false, false, false,
  ///                   true, true, false, false, false, false,
  ///                   false, false, false, false, false, false,
  ///                   false, false, false, false, false, false,
  ///                   false, false, false, false, false, false];
  ///
  /// // encrypt the initial configuration
  /// let states: Vec<Ciphertext> = states.into_iter().map(|x| client_key.encrypt(x)).collect();
  ///
  /// // build the board
  /// let mut board = Board::new(n_cols, states);
  /// ```
  pub fn new(n_cols: usize, states: Vec<Ciphertext>, client_key: ClientKey) -> Board {
    let n_rows = states.len() / n_cols;
    Board {
      dimensions: (n_rows, n_cols),
      states,
      client_key,
      tick_time: Duration::default(),
    }
  }

  /// update the board
  ///
  /// # Arguments
  ///
  /// * `server_key`: the server key
  /// * `zeros`: three encryption of `false`
  ///
  /// # Example
  ///
  /// ```
  /// use homomorphic_game_of_life::*;
  ///
  /// // numbers of rows and columns in the board
  /// let (n_rows, n_cols): (usize, usize) = (6,6);
  ///
  /// // generate the keys
  /// let (client_key, server_key) = gen_keys();
  ///
  /// // encrypt false three times
  /// let zeros = (client_key.encrypt(false), client_key.encrypt(false), client_key.encrypt(false));
  ///
  /// // initial configuration
  /// let states = vec![true, false, false, false, false, false,
  ///                   false, true, true, false, false, false,
  ///                   true, true, false, false, false, false,
  ///                   false, false, false, false, false, false,
  ///                   false, false, false, false, false, false,
  ///                   false, false, false, false, false, false];
  ///
  /// // encrypt the initial configuration
  /// let states: Vec<Ciphertext> = states.into_iter().map(|x| client_key.encrypt(x)).collect();
  ///
  /// // build the board
  /// let mut board = Board::new(n_cols, states);
  ///
  /// // update the board
  /// board.update(&server_key, &zeros);
  ///
  /// // decrypt and show the board
  /// for i in 0..n_rows {
  ///     println!("");
  ///     for j in 0..n_rows {
  ///         if client_key.decrypt(&board.states[i*n_cols+j]) {
  ///             print!("█");
  ///         } else {
  ///             print!("░");
  ///         }
  ///     }
  /// }
  /// println!("");
  /// ```
  pub fn update(
    &mut self,
    server_key: &ServerKey,
    zeros: &(Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
  ) {
    let nx = self.dimensions.0;
    let ny = self.dimensions.1;

    let mut new_states = Vec::<Ciphertext>::new();

    for k1 in (0..nx * ny).step_by(FPGA_BOOTSTRAP_PACKING) {
      let mut cell_p = Vec::<Ciphertext>::new();
      let mut neighbours_p = [
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
        Vec::<Ciphertext>::new(),
      ];
      let end = std::cmp::min(FPGA_BOOTSTRAP_PACKING, nx * ny - k1);

      for k2 in 0..end {
        let i = (k1 + k2) / ny;
        let j = (k1 + k2) % ny;

        let im = if i == 0 { nx - 1 } else { i - 1 };
        let ip = if i == nx - 1 { 0 } else { i + 1 };
        let jm = if j == 0 { ny - 1 } else { j - 1 };
        let jp = if j == ny - 1 { 0 } else { j + 1 };

        // get the neighbours, with periodic boundary conditions
        neighbours_p[0].push(self.states[im * ny + jm].clone());
        neighbours_p[1].push(self.states[im * ny + j].clone());
        neighbours_p[2].push(self.states[im * ny + jp].clone());
        neighbours_p[3].push(self.states[i * ny + jm].clone());
        neighbours_p[4].push(self.states[i * ny + jp].clone());
        neighbours_p[5].push(self.states[ip * ny + jm].clone());
        neighbours_p[6].push(self.states[ip * ny + j].clone());
        neighbours_p[7].push(self.states[ip * ny + jp].clone());

        cell_p.push(self.states[i * ny + j].clone());
      }

      is_alive(&server_key, cell_p, neighbours_p, zeros, &mut new_states);
    }

    // update the board
    self.states = new_states;
  }
}

// config structure
pub struct Config {
  pub wait_time_micros: u64,
  pub pixel_size: usize,
  pub dimensions: (usize, usize),
  pub col1: (f32, f32, f32),
  pub col2: (f32, f32, f32),
}

impl Config {
  pub fn read(fname: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let err_message = "Missing argument in the config file";

    // load the content
    let content = std::fs::read_to_string(fname)?;

    // separate the nubers
    let mut content = content.split(' ');

    // read the content
    Ok(Config {
      wait_time_micros: content.next().ok_or(err_message)?.parse::<u64>()?,
      pixel_size: content.next().ok_or(err_message)?.parse::<usize>()?,
      dimensions: (
        content.next().ok_or(err_message)?.parse::<usize>()?,
        content.next().ok_or(err_message)?.parse::<usize>()?,
      ),
      col1: (
        content.next().ok_or(err_message)?.parse::<f32>()?,
        content.next().ok_or(err_message)?.parse::<f32>()?,
        content.next().ok_or(err_message)?.parse::<f32>()?,
      ),
      col2: (
        content.next().ok_or(err_message)?.parse::<f32>()?,
        content.next().ok_or(err_message)?.parse::<f32>()?,
        content.next().ok_or(err_message)?.parse::<f32>()?,
      ),
    })
  }
}

/// read a state file, space- and newline-separated
///
/// The file must contain only 0s and 1s and all rows need to have the same length.
#[allow(clippy::type_complexity)]
pub fn read_csv(fname: &str) -> Result<((usize, usize), Vec<bool>), Box<dyn std::error::Error>> {
  // load the content
  let content = std::fs::read_to_string(fname)?;

  // separate in rows
  let content = content.split('\n').collect::<Vec<&str>>();
  let n_rows = content.len() - 1;

  // number of columns
  let n_cols = content[0].split(' ').count() - 1;

  // load the data
  let mut data = Vec::<bool>::new();
  for row in content {
    for el in row.split(' ') {
      if let Ok(x) = el.parse::<u8>() {
        data.push(x == 1)
      };
    }
  }

  Ok(((n_rows, n_cols), data))
}

impl Widget for Board {
  fn render(self, _area: Rect, buf: &mut Buffer) {
    for i in 0..self.dimensions.0 {
      for j in 0..self.dimensions.1 {
        if self
          .client_key
          .decrypt(&self.states[i * self.dimensions.1 + j])
        {
          buf
            .get_mut(2 + i as u16, 2 + j as u16)
            .set_style(Style::default().bg(Color::White));
        }
      }
    }

    buf.set_string(
      1,
      (self.dimensions.1 + 3) as u16,
      format!(
        "                                   Board update: {:?}",
        self.tick_time
      ),
      Style::default(),
    );
  }
}
