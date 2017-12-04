mod board;
mod movavg;

use board::Board;
use movavg::MovAvg;

const N_V_TABLES: usize = 17;
const ALPHA: f32 = 0.005;

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

fn get_val(vpos: VPos) -> f32 {
  vpos.iter().zip(unsafe { V_TABLES.iter() }).map(|(pos, table)| unsafe { table.get_unchecked(*pos as usize) }).sum()
}

fn main() {
  board::init();
  Board::print_spacing();
  print!("\n");
  let mut n_games = 0;

  let mut avg_score = MovAvg::new();
  avg_score.init(1000);

  'game: loop {
    n_games += 1;
    let mut board = Board(0);
    board.comp_move();
    let mut prev_vpos = board.vpos();

    loop {
      board.comp_move();

      let mut bestdir = -1;
      let mut bestvpos = [0; N_V_TABLES];
      let mut bestval = std::f32::NEG_INFINITY;
      let mut bestboard = Board(0);

      for dir in 0..4 {
        let newboard = board.slide(dir);
        if newboard == board {
          continue;
        }

        let vpos = newboard.vpos();
        // Optimizing out subtracting pre_score since it applies to all directions.
        let val = get_val(vpos) + newboard.game_score(0) as f32;
        if val > bestval {
          bestval = val;
          bestvpos = vpos;
          bestdir = dir;
          bestboard = newboard;
        }
      }

      // Dead
      if bestdir == -1 {
        avg_score.add(board.game_score(0));
        avg_score.drop();
        print!("\x1b[2A");
        board.print(0);
        println!("Avg score: {}", avg_score.avg());
        println!("Num games: {}", n_games);
        continue 'game;
      }

      // Learn
      let r = (bestboard.game_score(0) - board.game_score(0)) as f32;
      let adjust = (r + get_val(bestvpos) - get_val(prev_vpos)) * ALPHA;
      prev_vpos.iter().zip(unsafe { V_TABLES.iter_mut() })
                      .for_each(|(pos, table)| unsafe {
                        *table.get_unchecked_mut(*pos as usize) += adjust;
                      });

      // Execute best move
      prev_vpos = bestvpos;
      board = bestboard;
    }
  }
}
