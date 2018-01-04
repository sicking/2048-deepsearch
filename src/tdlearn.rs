mod board;
mod movavg;

use board::Board;
use movavg::MovAvg;

const N_V_TABLES: usize = 17;
const ALPHA_START: f32 = 0.0025;
const ALPHA_DECREASE: f32 = 5.0;
const ALPHA_RATE: f32 = 300000.0;
const EXPLORE_DECREASE_FACTOR: f32 = 2.0;

static mut V_TABLES : [[f32; 65536]; N_V_TABLES] = [[0f32; 65536]; N_V_TABLES];

type VPos = [u16; N_V_TABLES];

impl Board {
  fn vpos(self) -> VPos {
    let mut res : VPos = [0u16; N_V_TABLES];
    // First the horizontal positions
    for i in 0..4 {
      res[i] = ((self.0 >> (16 * i)) & 0xffff) as u16;
    }

    // Then vertical
    let t = self.transpose();
    for i in 0..4 {
      res[i+4] = ((t.0 >> (16 * i)) & 0xffff) as u16;
    }

    // Then squares
    let mut b1 = self.0;
    let mut b2 = self.0 >> 8;
    let mut n = 8;
    for _ in 0..3 {
      for _ in 0..3 {
        res[n] = ((b1 & 0xff) | (b2 & 0xff00)) as u16;
        n += 1;
        b1 >>= 4;
        b2 >>= 4;
      }
      b1 >>= 4;
      b2 >>= 4;
    }

    res
  }
}

#[cfg(not(feature = "best-symmetry"))]
fn get_val(board: Board) -> (VPos, f32) {
  let vpos = board.vpos();
  (vpos, vpos.iter().zip(unsafe { V_TABLES.iter() }).map(|(pos, table)| unsafe { table.get_unchecked(*pos as usize) }).sum())
}

#[cfg(feature = "best-symmetry")]
fn get_val(board: Board) -> (VPos, f32) {
  let mut bestvpos = [0; N_V_TABLES];
  let mut bestval = std::f32::NEG_INFINITY;
  for symm in board.symmetries() {
    let vpos = symm.vpos();
    let val = vpos.iter().zip(unsafe { V_TABLES.iter() }).map(|(pos, table)| unsafe { table.get_unchecked(*pos as usize) }).sum();
    if val > bestval {
      bestval = val;
      bestvpos = vpos;
    }
  }
  (bestvpos, bestval)
}

fn main() {
  board::init();
  Board::print_spacing();
  print!("\n\n");
  let mut n_games = 0;

  let mut avg_score = MovAvg::new();
  avg_score.init(1000);

  loop {
    let alpha = ALPHA_START / ALPHA_DECREASE.powf((n_games as f32) / ALPHA_RATE);
    n_games += 1;
    let mut board = Board(0);
    board.comp_move();
    let (mut prev_vpos, mut prev_val) = get_val(board);

    loop {
      board.comp_move();

      let mut bestdir = -1;
      let mut bestvpos = [0; N_V_TABLES];
      let mut bestboard = Board(0);
      let mut bestval = std::f32::NEG_INFINITY;

      let rand_move = rng((n_games as f32 * EXPLORE_DECREASE_FACTOR) as u32) == 0;
      if rand_move {
        let mut ndir = 0;
        let mut allowed_dirs = [0; 4];

        for dir in 0..4 {
          let newboard = board.slide(dir);
          if newboard != board {
            allowed_dirs[ndir] = dir;
            ndir += 1;
          }
        }

        if ndir > 0 {
          bestdir = allowed_dirs[rng(ndir as u32) as usize];
          bestboard = board.slide(bestdir);
          let (vpos, val) = get_val(bestboard);
          bestvpos = vpos;
          bestval = val;
        }

      } else {
        for dir in 0..4 {
          let newboard = board.slide(dir);
          if newboard == board {
            continue;
          }

          // Optimizing out adding 'r' since it's 1 for every direction.
          let (vpos, val) = get_val(newboard);
          if val > bestval {
            bestval = val;
            bestvpos = vpos;
            bestdir = dir;
            bestboard = newboard;
          }
        }
      }

      // Learn
      if !rand_move {
        let exp_value = if bestdir == -1 {
                          0.0
                        }
                        else {
                          1.0 + bestval
                        };
        let adjust = (exp_value - prev_val) * alpha;
        prev_vpos.iter().zip(unsafe { V_TABLES.iter_mut() })
                        .for_each(|(pos, table)| unsafe {
                          *table.get_unchecked_mut(*pos as usize) += adjust;
                        });
      }

      // Dead
      if bestdir == -1 {
        break;
      }

      // Execute best move
      prev_vpos = bestvpos;
      prev_val = bestval;
      board = bestboard;
    }

    avg_score.add(board.game_score(0));
    avg_score.drop();
    if (n_games % 2000) == 0 {
      print!("\x1b[2A");
      board.print(0);
      println!("Avg score: {}  ", avg_score.avg());
      println!("Num games: {}", n_games);
    }
  }
}

static mut SEED: u32 = 0x17014711;
fn rng(max: u32) -> u32 {
  let mut x = unsafe { SEED };
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  unsafe { SEED = x; }
  (x % max) as u32
}
