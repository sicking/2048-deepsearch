extern crate rand;
extern crate getch;

use rand::{Rng, StdRng};
use std::time::Instant;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Board(u64);

impl Board {
  #[allow(dead_code)]
  fn print_spacing() {
    for _ in 0..10 { println!(); }
  }
  fn print(self, fours: i32) {
    let nums = ["   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"];
    print!("\x1b[10A+---+---+---+---+\n");
    for n in 0..16 {
      print!("|{}", nums[self.get_tile(15-n) as usize]);
      if (n % 4) == 3 {
        print!("|\n+---+---+---+---+\n");
      }
    }
    println!("Score: {}      ", self.game_score(fours));
  }

  fn empty(self) -> i32 {
    fn empty_debug(board: Board) -> i32 {
      let mut empty = 0;
      for i in 0..16 {
        if board.get_tile(i) == 0 {
          empty += 1;
        }
      }
      empty
    }

    // Below can't handle this case.
    debug_assert_ne!(self.0, 0);

    let b = !(self.0 | (self.0 >> 1) | (self.0 >> 2) | (self.0 >> 3)) & 0x1111_1111_1111_1111u64;
    let n1 = b + (b >> 16) + (b >> 32) + (b >> 48);
    let n2 = (n1 + (n1 >> 4) + (n1 >> 8) + (n1 >> 12)) & 0xf;
    debug_assert!((n2 as i32) == empty_debug(self));
    n2 as i32
  }

  fn get_tile(self, tile: i32) -> i32 {
    debug_assert!(tile >= 0 && tile < 16);
    ((self.0 >> (tile * 4)) & 0xf) as i32
  }

  fn set_tile(&mut self, tile: i32, val: i32) {
    debug_assert_eq!(self.get_tile(tile), 0);
    self.0 |= (val as u64) << (tile * 4);
  }

  fn comp_move(&mut self, rng: &mut Rng) -> i32 {
    // self.empty() can't handle the completely-empty case.
    debug_assert!(self.0 == 0 || self.empty() > 0);
    let size = if self.0 == 0 { 16 } else { self.empty() };
    let mut n = (rng.next_f32() * (size as f32)).floor() as i32;
    debug_assert!(self.0 == 0 || n < self.empty());
    let mut pos = -1;
    while n >= 0 {
      pos += 1;
      if self.get_tile(pos) == 0 {
        n -= 1;
      }
    }
    let four = rng.next_f32() < 0.1;
    self.set_tile(pos, if four { 2 } else { 1 });
    if four { 1 } else { 0 }
  }

  fn slide(self, dir: i32) -> Board {
    match dir {
      0 => self.slide_right(),
      1 => self.slide_down(),
      2 => self.slide_left(),
      3 => self.slide_up(),
      _ => panic!("unknown direction"),
    }
  }

  fn slide_down(self) -> Board {
    let t = self.transpose();
    Board(unsafe {
      (SLIDE_TABLE[((t.0 >> 0) & 0xffff) as usize] as u64) << 0 |
      (SLIDE_TABLE[((t.0 >> 16) & 0xffff) as usize] as u64) << 16 |
      (SLIDE_TABLE[((t.0 >> 32) & 0xffff) as usize] as u64) << 32 |
      (SLIDE_TABLE[((t.0 >> 48) & 0xffff) as usize] as u64) << 48
      }).transpose()
  }

  fn slide_up(self) -> Board {
    let t = self.transpose2();
    Board(unsafe {
      (SLIDE_TABLE[((t.0 >> 0) & 0xffff) as usize] as u64) << 0 |
      (SLIDE_TABLE[((t.0 >> 16) & 0xffff) as usize] as u64) << 16 |
      (SLIDE_TABLE[((t.0 >> 32) & 0xffff) as usize] as u64) << 32 |
      (SLIDE_TABLE[((t.0 >> 48) & 0xffff) as usize] as u64) << 48
      }).transpose2()
  }

  fn slide_right(self) -> Board {
    Board(unsafe {
      (SLIDE_TABLE[((self.0 >> 0) & 0xffff) as usize] as u64) << 0 |
      (SLIDE_TABLE[((self.0 >> 16) & 0xffff) as usize] as u64) << 16 |
      (SLIDE_TABLE[((self.0 >> 32) & 0xffff) as usize] as u64) << 32 |
      (SLIDE_TABLE[((self.0 >> 48) & 0xffff) as usize] as u64) << 48
      })
  }

  fn slide_left(self) -> Board {
    let t = self.horiz_flip();
    Board(unsafe {
      (SLIDE_TABLE[((t.0 >> 0) & 0xffff) as usize] as u64) << 0 |
      (SLIDE_TABLE[((t.0 >> 16) & 0xffff) as usize] as u64) << 16 |
      (SLIDE_TABLE[((t.0 >> 32) & 0xffff) as usize] as u64) << 32 |
      (SLIDE_TABLE[((t.0 >> 48) & 0xffff) as usize] as u64) << 48
      }).horiz_flip()
  }

  fn game_score(self, fours: i32) -> i32 {
    let mut score: i32 = 0;
    for pos in 0..16 {
      let val = ((self.0 >> (pos*4)) & 0xf) as i32;
      if val >= 2 {
        score += (val - 1) * (1 << val);
      }
    }
    score - (fours * 4)
  }

  fn transpose(self) -> Board {
    // 0123      048c      0 3 6 9
    // 4567  ->  159d  =  -3 0 3 6
    // 89ab      26ae     -6-3 0 3
    // cdef      37bf     -9-6-3 0


    //  0 0 6 6
    //  0 0 6 6
    // -6-6 0 0
    // -6-6 0 0

    let a1 = self.0 & 0xf0f0_0f0f_f0f0_0f0f_u64;
    //                  0_2_ _5_7 8_a_ _d_f
    let a2 = self.0 & 0x0000_f0f0_0000_f0f0_u64;
    //                  ____ 4_6_ ____ c_e_
    let a3 = self.0 & 0x0f0f_0000_0f0f_0000_u64;
    //                  _1_3 ____ _9_b ____
    let a = a1 | (a2 << 12) | (a3 >> 12);
    //                  0426 1537 8cae 9dbf

    let b1 = a      & 0xff00_ff00_00ff_00ff_u64;
    //                  04__ 15__ __ae __bf
    let b2 = a      & 0x00ff_00ff_0000_0000_u64;
    //                  __26 __37 ____ ____
    let b3 = a      & 0x0000_0000_ff00_ff00_u64;
    //                  ____ ____ 8c__ 9d__

    Board(b1 | (b2 >> 24) | (b3 << 24))
    //                  048c 159d 26ae 37bf
  }

  fn transpose2(self) -> Board {
    // 0123      fb73      15 10   5   0
    // 4567  ->  ea62   =  10  5   0  -5
    // 89ab      d951       5  0  -5 -10
    // cdef      c840       0 -5 -10 -15

    // 10 10   0    0
    // 10 10   0    0
    //  0  0  -10 -10
    //  0  0  -10 -10


    let a1 = self.0 & 0x0f0f_f0f0_0f0f_f0f0_u64;
    let a2 = self.0 & 0x0000_0f0f_0000_0f0f_u64;
    let a3 = self.0 & 0xf0f0_0000_f0f0_0000_u64;
    let a = a1 | (a2 << 20) | (a3 >> 20);

    let b1 = a      & 0x00ff_00ff_ff00_ff00_u64;
    let b2 = a      & 0xff00_ff00_0000_0000_u64;
    let b3 = a      & 0x0000_0000_00ff_00ff_u64;

    Board(b1 | (b2 >> 40) | (b3 << 40))
  }

  fn horiz_flip(self) -> Board {
    // 0123      3210      3  1 -1 -3
    // 4567  ->  7654   =  3  1 -1 -3
    // 89ab      ba98      3  1 -1 -3
    // cdef      fedc      3  1 -1 -3

    let a1 = self.0 & 0xf000_f000_f000_f000_u64;
    let a2 = self.0 & 0x0f00_0f00_0f00_0f00_u64;
    let a3 = self.0 & 0x00f0_00f0_00f0_00f0_u64;
    let a4 = self.0 & 0x000f_000f_000f_000f_u64;

    Board(a1 >> 12 | a2 >> 4 | a3 << 4 | a4 << 12)
  }


  fn score(self) -> f32 {
    let trans = self.transpose();
    unsafe {
      SCORE_TABLE[((self.0 >> 0) & 0xffff) as usize] +
      SCORE_TABLE[((self.0 >> 16) & 0xffff) as usize] +
      SCORE_TABLE[((self.0 >> 32) & 0xffff) as usize] +
      SCORE_TABLE[((self.0 >> 48) & 0xffff) as usize] +
      SCORE_TABLE[((trans.0 >> 0) & 0xffff) as usize] +
      SCORE_TABLE[((trans.0 >> 16) & 0xffff) as usize] +
      SCORE_TABLE[((trans.0 >> 32) & 0xffff) as usize] +
      SCORE_TABLE[((trans.0 >> 48) & 0xffff) as usize]
    }
  }

}

fn ai_comp_move(board: Board, depth: i32) -> f32 {
  if depth <= 0 {
    return board.score();
  }

  let empty = board.empty();
  debug_assert!(empty != 0);
  let mut score = 0f32;
  for tile in 0..16 {
    if board.get_tile(tile) == 0 {
      let mut new_board = board;
      new_board.set_tile(tile, 1);
      score += ai_player_move(new_board, depth) * 0.9f32;
      new_board = board;
      new_board.set_tile(tile, 2);
      score += ai_player_move(new_board, depth) * 0.1f32;
    }
  }

  score / (empty as f32)
}

fn ai_player_move(board: Board, depth: i32)  -> f32 {
  let mut score = 0f32;

  for dir in 0..4 {
    let new_board = board.slide(dir);
    if new_board == board {
      continue;
    }

    let move_score = ai_comp_move(new_board, depth - 1);
    if move_score > score {
      score = move_score;
    }
  }

  score
}

const SCORE_LOST_PENALTY : f32 = 200000.0f32;
const SCORE_MONOTONICITY_POWER : f32 = 4.0f32;
const SCORE_MONOTONICITY_WEIGHT : f32 = 47.0f32;
const SCORE_SUM_POWER : f32 = 3.5f32;
const SCORE_SUM_WEIGHT : f32 = 11.0f32;
const SCORE_MERGES_WEIGHT : f32 = 700.0f32;
const SCORE_EMPTY_WEIGHT : f32 = 270.0f32;

static mut SCORE_TABLE : [f32; 65536] = [0f32; 65536];
static mut SLIDE_TABLE : [u16; 65536] = [0u16; 65536];

fn init_score_table() {
  for n in 0..65536 {
    let vals = [(n >> 0) & 0xf,
                (n >> 4) & 0xf,
                (n >> 8) & 0xf,
                (n >> 12) & 0xf];

    let mut sum = 0f32;
    let mut empty = 0;
    let mut merges = 0;
    let mut counter = 0;
    let mut prev = 0;
    for rank in vals.iter() {
      sum += (*rank as f32).powf(SCORE_SUM_POWER);
      if *rank == 0 {
        empty += 1;
      } else {
        if prev == *rank {
          counter += 1;
        } else if counter > 0 {
          merges += 1 + counter;
          counter = 0;
        }
        prev = *rank;
      }
    }
    if counter > 0 {
      merges += 1 + counter;
    }

    let mut monotonicity_left = 0f32;
    let mut monotonicity_right = 0f32;
    for j in 1..4 {
      let i = j as usize;
      if vals[i-1] > vals[i] {
        monotonicity_left += (vals[i-1] as f32).powf(SCORE_MONOTONICITY_POWER) - (vals[i] as f32).powf(SCORE_MONOTONICITY_POWER);
      } else {
        monotonicity_right += (vals[i] as f32).powf(SCORE_MONOTONICITY_POWER) - (vals[i-1] as f32).powf(SCORE_MONOTONICITY_POWER);
      }
    }

    let score = SCORE_LOST_PENALTY +
                SCORE_EMPTY_WEIGHT * (empty as f32) +
                SCORE_MERGES_WEIGHT * (merges as f32) -
                SCORE_MONOTONICITY_WEIGHT * if monotonicity_left < monotonicity_right { monotonicity_left } else { monotonicity_right } -
                SCORE_SUM_WEIGHT * sum;
    unsafe { SCORE_TABLE[n] = score; }

    let mut res = vals[0] as u16;
    let mut merge_val = vals[0];
    let mut dest_pos = if merge_val == 0 { -4 } else { 0 };
    for pos in 1..4 {
      let val = vals[pos as usize];

      if val == 0 {
        // do nothing
      } else if val == merge_val {
        res += (1 as u16) << dest_pos;
        merge_val = 0;
      } else {
        dest_pos += 4;
        merge_val = val;
        res |= (val as u16) << dest_pos;
      }
    }
    unsafe { SLIDE_TABLE[n] = res; }

  }
}

fn ai_play(rng: &mut Rng, print: bool) -> i32 {

  let mut board = Board(0);
  let mut fours = 0;
  fours += board.comp_move(rng);
  fours += board.comp_move(rng);

  loop {
    if print {
      board.print(fours);
    }

    let mut bestexp = 0f32;
    let mut bestdir = -1;

    for dir in 0..4 {
      let new_board = board.slide(dir);
      if new_board == board {
        continue;
      }

      let exp = ai_comp_move(new_board, 2);
      if exp > bestexp {
        bestexp = exp;
        bestdir = dir;
      }
    }

    if bestdir == -1 {
      return board.game_score(fours);
    }

    board = board.slide(bestdir);
    fours += board.comp_move(rng);
  }
}


#[allow(dead_code)]
fn play_manual() -> std::result::Result<(), std::io::Error> {
  Board::print_spacing();

  let mut board = Board(0);

  let mut rng : rand::ThreadRng = rand::thread_rng();

  let mut fours = 0;
  fours += board.comp_move(&mut rng);
  fours += board.comp_move(&mut rng);

  let io = getch::Getch::new()?;

  loop {
    board.print(fours);

    let new_board;
    match io.getch()? as char {
      'q' => break,
      'w' => new_board = board.slide_up(),
      's' => new_board = board.slide_down(),
      'd' => new_board = board.slide_right(),
      'a' => new_board = board.slide_left(),
      't' => { board = board.transpose(); continue },
      'T' => { board = board.transpose2(); continue },
      'f' => { board = board.horiz_flip(); continue },
      _ => continue,
    }

    if new_board == board {
      continue;
    }

    board = new_board;
    fours += board.comp_move(&mut rng);
  }

  Ok(())
}


#[allow(dead_code)]
fn ai_play_multi_games() {
  let seed: &[_] = &[1, 2, 3, 4, 5];
  let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
  //let mut rng : rand::ThreadRng = rand::thread_rng();

  let now = Instant::now();
  let n = 50;
  let mut tot_score = 0;
  for _ in 0..n {
    let score = ai_play(&mut rng, false);
    tot_score += score;
    println!("Score: {}", score);
  }

  let elapsed = now.elapsed();

  println!("Average score: {}, time: {}", (tot_score as f32) / (n as f32),
             elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64) / 1_000_000_000f64);  
}


fn main() {
  init_score_table();

  //play_manual().unwrap();
  ai_play_multi_games();
}
