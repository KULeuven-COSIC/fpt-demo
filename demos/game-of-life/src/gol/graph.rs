use tfhe::boolean::prelude::*;

pub use ggez::{conf, event, graphics, Context, GameResult};

use super::board::*;

pub struct MainState {
  board: Board,
  // time_step: Duration,
  first_frame: bool,
  pixel_size: usize,
  col1: (f32, f32, f32),
  col2: (f32, f32, f32),
  server_key: ServerKey,
  zeros: (Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
  client_key: ClientKey,
}

impl MainState {
  pub fn new(
    board: Board,
    config: &Config,
    server_key: ServerKey,
    zeros: (Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
    client_key: ClientKey,
  ) -> Result<MainState, Box<dyn std::error::Error>> {
    Ok(MainState {
      board,
      // time_step: Duration::from_micros(config.wait_time_micros),
      first_frame: true,
      pixel_size: config.pixel_size,
      col1: config.col1,
      col2: config.col2,
      server_key,
      zeros,
      client_key,
    })
  }
}

impl event::EventHandler<ggez::GameError> for MainState {
  fn update(&mut self, _ctx: &mut Context) -> GameResult {
    if self.first_frame {
      self.first_frame = false;
    } else {
      // sleep(self.time_step);
      use std::time::Instant;
      let now = Instant::now();
      self.board.update(&self.server_key, &self.zeros);
      let elapsed = now.elapsed();
      println!("Elapsed: {:.2?}", elapsed);
    }
    Ok(())
  }

  fn draw(&mut self, mut ctx: &mut Context) -> GameResult {
    // clear the window
    graphics::clear(ctx, [self.col1.0, self.col1.1, self.col1.2, 1.].into());

    for i in 0..self.board.dimensions.0 {
      for j in 0..self.board.dimensions.1 {
        if self
          .client_key
          .decrypt(&self.board.states[i * self.board.dimensions.1 + j])
        {
          let pixel = graphics::MeshBuilder::new()
            .rectangle(
              graphics::DrawMode::Fill(graphics::FillOptions::DEFAULT),
              graphics::Rect::new(0., 0., self.pixel_size as f32, self.pixel_size as f32),
              graphics::Color::new(self.col2.0, self.col2.1, self.col2.2, 1.),
            )
            .unwrap()
            .build(&mut ctx)
            .unwrap();
          graphics::draw(
            ctx,
            &pixel,
            graphics::DrawParam::new().offset([
              -((j * self.pixel_size) as f32),
              -((i * self.pixel_size) as f32),
            ]),
          )?;
        }
      }
    }

    graphics::present(ctx)?;
    Ok(())
  }
}
