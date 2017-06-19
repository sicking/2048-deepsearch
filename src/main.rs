extern crate rand;
extern crate getch;
extern crate byteorder;
extern crate getopts;
extern crate futures;
extern crate futures_cpupool;

use std::time::Instant;
use std::collections::HashMap;
use byteorder::{NativeEndian, WriteBytesExt, ReadBytesExt};
use std::fs::File;
use std::io::BufWriter;
use std::result::Result;
use getopts::Options;
use futures::{Future, future};
use futures_cpupool::CpuPool;
use std::cell::RefCell;
use std::cell::Cell;

thread_local!(static HASH: (RefCell<HashMap<Board, (i32, f32, f32)>>, Cell<u32>) = (RefCell::new(HashMap::new()), Cell::new(0)));

fn ai_play(until: i32, print: bool, filename: Option<&String>) -> Result<i32, std::io::Error> {
  let mut board = Board(0);
  let mut fours = 0;
  fours += board.comp_move();
  fours += board.comp_move();

  let mut file = None;
  if let Some(fname) = filename {
    file = Some(BufWriter::new(File::create(fname)?));
  }

  let pool = CpuPool::new_num_cpus();

  if print {
    Board::print_spacing();
  }


  let mut gen = 0;
  let mut state = PlayState::ZeroProbDeath;

  loop {
    gen += 1;

    if print {
      board.print(fours);
    }

    let mut bestdir = -1;
    let mut searched_depth = 0;
    let mut bestexp = 0f32;
    let mut best_end_prob = 1f32;
    let mut depth: u8;

    let mut searches = 0;

    while {
      depth = match state {
        PlayState::ZeroProbDeath => std::cmp::max(3, board.distinct() - 4),
        PlayState::LowProbDeath => std::cmp::max(3, board.distinct() - 2),
        PlayState::HighPropDeath => std::cmp::max(3, board.distinct()),
        PlayState::VeryHighProbDeath => 17,
      };
      depth > searched_depth } {

      bestdir = -1;
      bestexp = 0.0;
      best_end_prob = 1.0;

      let mut results = Vec::new();
      for dir in 0..4 {
        let new_board = board.slide(dir);
        if new_board == board {
          continue;
        }

        results.push(async_ai_comp_move(&pool, gen, new_board, depth, 1f32).map(move |(exp, end_prob)| (dir, exp, end_prob)));
      }

      let res = future::join_all(results).wait().unwrap();

      for (dir, exp, end_prob) in res {
        if exp > bestexp {
          bestexp = exp;
          bestdir = dir as i32;
          best_end_prob = end_prob;
        }
      }

      searched_depth = depth;
      searches += 1;

      state = PlayState::from_prob(best_end_prob);

      if print {
        print!("Death prob: {:.9}, Depth: {} State: {:?}          \n\x1b[1A", best_end_prob, depth, state);
      }
    }

    if let Some(ref mut f) = file.as_mut() {
      f.write_u64::<NativeEndian>(board.0)?;
      f.write_i32::<NativeEndian>(fours)?;
      f.write_f32::<NativeEndian>(bestexp)?;
      f.write_f32::<NativeEndian>(best_end_prob)?;
      f.write_i8(bestdir as i8)?;
      f.write_u8(searched_depth as u8)?;
      f.write_u8(searches as u8)?;
    }

    if (until > 0 && board.max_val() >= until) ||
       bestdir == -1 {
      break;
    }

    board = board.slide(bestdir);
    fours += board.comp_move();
  }

  if !print {
    println!("Score: {}", board.game_score(fours));
  } else {
    println!();
  }

  Ok(board.game_score(fours))
}

fn async_ai_comp_move(pool: &CpuPool, gen: u32, board: Board, depth: u8, prob: f32) -> future::BoxFuture<(f32, f32), ()> {

  // Don't deal with this case since it'd be hard to return a Future of the right type
  debug_assert!(depth > 0 && prob > 0.0001);

  let empty = board.empty();
  debug_assert!(empty != 0);

  let prob1 = prob / (empty as f32) * 0.9;
  let prob2 = prob / (empty as f32) * 0.1;

  let mut futures = Vec::new();

  for tile in 0..16 {
    if board.get_tile(tile) == 0 {
      futures.push(pool.spawn_fn(move || -> Result<(f32, f32), ()> {
        HASH.with(|&(ref hash_cell, ref hash_gen)| {
          let mut hash = hash_cell.borrow_mut();
          // Only clear if we're processing a new move
          if hash_gen.get() != gen {
            hash.clear();
            hash_gen.set(gen);
          }
          let (score, end_prob) = ai_player_move(board.set_tile(tile, 1), depth, &mut *hash, prob1);
          Ok((score * 0.9, end_prob * 0.9))
        })
      }));
      futures.push(pool.spawn_fn(move || -> Result<(f32, f32), ()> {
        HASH.with(|&(ref hash_cell, ref hash_gen)| {
          let mut hash = hash_cell.borrow_mut();
          // Only clear if we're processing a new move
          if hash_gen.get() != gen {
            hash.clear();
            hash_gen.set(gen);
          }
          let (score, end_prob) = ai_player_move(board.set_tile(tile, 2), depth, &mut *hash, prob2);
          Ok((score * 0.1, end_prob * 0.1))
        })
      }));
    }
  }

  future::join_all(futures).map(move |results| -> (f32, f32) {
    let mut score = 0f32;
    let mut end_prob = 0f32;

    for (move_score, move_end_prob) in results {
      score += move_score;
      end_prob += move_end_prob;
    }

    score /= empty as f32;
    end_prob /= empty as f32;
    (score, end_prob)
  }).boxed()
}

fn ai_comp_move(board: Board, depth: u8, hash: &mut HashMap<Board, (i32, f32, f32)>, prob: f32) -> (f32, f32) {
  if depth <= 0 || prob < 0.0001 {
    return (board.score(), 0f32);
  }

  if let Some(entry) = hash.get(&board) {
    let (hash_depth, score, end_prob) = *entry;
    if hash_depth >= depth as i32 {
      return (score, end_prob);
    }
  }

  let empty = board.empty();
  debug_assert!(empty != 0);

  let prob1 = prob / (empty as f32) * 0.9;
  let prob2 = prob / (empty as f32) * 0.1;

  let mut score = 0f32;
  let mut end_prob = 0f32;
  for tile in 0..16 {
    if board.get_tile(tile) == 0 {
      let (move_score_1, move_end_prob_1) = ai_player_move(board.set_tile(tile, 1), depth, hash, prob1);
      let (move_score_2, move_end_prob_2) = ai_player_move(board.set_tile(tile, 2), depth, hash, prob2);
      score += move_score_1 * 0.9 + move_score_2 * 0.1;
      end_prob += move_end_prob_1 * 0.9 + move_end_prob_2 * 0.1;
    }
  }

  score /= empty as f32;
  end_prob /= empty as f32;

  hash.insert(board, (depth as i32, score, end_prob));

  (score, end_prob)
}

fn ai_player_move(board: Board, depth: u8, hash: &mut HashMap<Board, (i32, f32, f32)>, prob: f32)  -> (f32, f32) {
  let mut score = 0f32;
  let mut end_prob = 1f32;

  for dir in 0..4 {
    let new_board = board.slide(dir);
    if new_board == board {
      continue;
    }

    let (move_score, move_end_prob) = ai_comp_move(new_board, depth - 1, hash, prob);
    if move_score > score {
      score = move_score;
      end_prob = move_end_prob;
    }
  }

  (score, end_prob)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Board(u64);

impl Board {
  fn print_spacing() {
    print!("\n\n\n\n\n\n\n\n\n\n\n\n\x1b[2A");
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
    println!("Score: {}", self.game_score(fours));
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

  fn distinct(self) -> u8 {
    let mut bits = 0usize;
    let mut b = self.0;
    while b != 0 {
      bits |= 1 << (b & 0xf);
      b >>= 4;
    }

    // Don't count empty tiles.
    bits &= !1usize;

    let mut count = 0;
    while bits != 0 {
        bits &= bits - 1;
        count += 1;
    }
    count
  }

  fn max_val(self) -> i32 {
    let mut max = 0;
    for pos in 0..16 {
      max = std::cmp::max(max, self.get_tile(pos));
    }
    max
  }

  fn get_tile(self, tile: i32) -> i32 {
    debug_assert!(tile >= 0 && tile < 16);
    ((self.0 >> (tile * 4)) & 0xf) as i32
  }

  fn set_tile(self, tile: i32, val: i32) -> Board {
    debug_assert_eq!(self.get_tile(tile), 0);
    Board(self.0 | (val as u64) << (tile * 4))
  }

  fn comp_move(&mut self) -> i32 {
    // self.empty() can't handle the completely-empty case.
    debug_assert!(self.0 == 0 || self.empty() > 0);
    let size = if self.0 == 0 { 16 } else { self.empty() };
    let mut n = rng(size);
    debug_assert!(self.0 == 0 || n < self.empty());
    let mut pos = -1;
    while n >= 0 {
      pos += 1;
      if self.get_tile(pos) == 0 {
        n -= 1;
      }
    }
    let four = rng(10) == 0;
    self.0 = self.set_tile(pos, if four { 2 } else { 1 }).0;
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
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 48) & 0xffff) as usize) as u64) << 48
      }).transpose()
  }

  fn slide_up(self) -> Board {
    let t = self.transpose();
    Board(unsafe {
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 48) & 0xffff) as usize) as u64) << 48
      }).transpose()
  }

  fn slide_right(self) -> Board {
    Board(unsafe {
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 48) & 0xffff) as usize) as u64) << 48
      })
  }

  fn slide_left(self) -> Board {
    Board(unsafe {
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 48) & 0xffff) as usize) as u64) << 48
      })
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
    let a1 = self.0 & 0xf0f0_0f0f_f0f0_0f0f_u64;
    let a2 = self.0 & 0x0000_f0f0_0000_f0f0_u64;
    let a3 = self.0 & 0x0f0f_0000_0f0f_0000_u64;
    let a = a1 | (a2 << 12) | (a3 >> 12);
    let b1 = a      & 0xff00_ff00_00ff_00ff_u64;
    let b2 = a      & 0x00ff_00ff_0000_0000_u64;
    let b3 = a      & 0x0000_0000_ff00_ff00_u64;
    Board(b1 | (b2 >> 24) | (b3 << 24))
  }

  fn score(self) -> f32 {
    let trans = self.transpose();
    unsafe {
      SCORE_TABLE.get_unchecked(((self.0 >> 0) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((self.0 >> 16) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((self.0 >> 32) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((self.0 >> 48) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((trans.0 >> 0) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((trans.0 >> 16) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((trans.0 >> 32) & 0xffff) as usize) +
      SCORE_TABLE.get_unchecked(((trans.0 >> 48) & 0xffff) as usize)
    }
  }
}

#[derive(Debug)]
enum PlayState {
  ZeroProbDeath,
  LowProbDeath,
  HighPropDeath,
  VeryHighProbDeath,
}

impl PlayState {
  fn from_prob(prob: f32) -> PlayState {
    if prob > 0.05 {
      PlayState::VeryHighProbDeath
    } else if prob > 0.001 {
      PlayState::HighPropDeath
    } else if prob > 0.0 {
      PlayState::LowProbDeath
    } else {
      PlayState::ZeroProbDeath
    }
  }
}

const SCORE_LOST_PENALTY : f32 = 200000.0f32;
const SCORE_MONOTONICITY_POWER : f32 = 4.0f32;
const SCORE_MONOTONICITY_WEIGHT : f32 = 47.0f32;
const SCORE_SUM_POWER : f32 = 3.5f32;
const SCORE_SUM_WEIGHT : f32 = 11.0f32;
const SCORE_MERGES_WEIGHT : f32 = 700.0f32;
const SCORE_EMPTY_WEIGHT : f32 = 270.0f32;

static mut SCORE_TABLE : [f32; 65536] = [0f32; 65536];
static mut SLIDE_RIGHT_TABLE : [u16; 65536] = [0u16; 65536];
static mut SLIDE_LEFT_TABLE : [u16; 65536] = [0u16; 65536];

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
        if (res >> dest_pos) & 0xf != 15 {
          res += (1 as u16) << dest_pos;
        }
        merge_val = 0;
      } else {
        dest_pos += 4;
        merge_val = val;
        res |= (val as u16) << dest_pos;
      }
    }
    fn reverse_row(row: u16) -> u16 {
      ((row & 0xf000) >> 12) |
      ((row & 0x0f00) >> 4) |
      ((row & 0x00f0) << 4) |
      ((row & 0x000f) << 12)
    }
    unsafe { SLIDE_RIGHT_TABLE[n] = res; }
    unsafe { SLIDE_LEFT_TABLE[reverse_row(n as u16) as usize] = reverse_row(res); }

  }
}

fn replay(filename: &str) -> Result<(), std::io::Error> {

  struct GameState {
    board: Board,
    fours: i32,
    bestexp: f32,
    best_end_prob: f32,
    bestdir: i8,
    depth: u8,
    searches: u8,
  }

  let mut states: Vec<GameState>;

  {
    let mut extra_searches = 0;
    let mut death_sum = 0f32;
    let mut life_prob = 1f64;
    let mut f = File::open(filename)?;
    let n = f.metadata()?.len() / 23;
    states = Vec::with_capacity(n as usize);
    for _ in 0..n {
      states.push(GameState {
                    board: Board(f.read_u64::<NativeEndian>()?),
                    fours: f.read_i32::<NativeEndian>()?,
                    bestexp: f.read_f32::<NativeEndian>()?,
                    best_end_prob: f.read_f32::<NativeEndian>()?,
                    bestdir: f.read_i8()?,
                    depth: f.read_u8()?,
                    searches: f.read_u8()?,
                  });
      extra_searches += states.last().unwrap().searches - 1;
      if states.last().unwrap().best_end_prob != 1.0 {
        death_sum += states.last().unwrap().best_end_prob;
        life_prob *= 1.0 - (states.last().unwrap().best_end_prob as f64);
      }
    }

    println!("Total moves: {}", n);
    println!("Redone searches: {}", extra_searches);
    println!("Death probability sum: {}", death_sum);
    println!("Death probability: {}", 1.0 - life_prob);
  }

  let io = getch::Getch::new()?;

  let mut pos: isize = 0;

  Board::print_spacing();
  print!("\n\n\n\n\n\n");

  loop {
    let state = &states[pos as usize];

    print!("\x1b[6A");
    state.board.print(state.fours);
    println!("Move: {}      ", pos);
    println!("Expected heuristic score: {:.2}      ", state.bestexp);
    println!("Probability of death: {:.9} ({:?})       ", state.best_end_prob, PlayState::from_prob(state.best_end_prob));
    println!("Searched depth: {}  ", state.depth);
    println!("Number of searches: {}  ", state.searches);
    if state.bestdir == -1 {
      println!("End of game.         ");
    } else {
      assert!(state.bestdir <= 3);
      println!("Decided direction: {}", "RDLU".chars().nth(state.bestdir as usize).unwrap());
    }

    pos += match io.getch()? as char {
      'q' => break,
      'd' => 1,
      's' => -1,
      'f' => 10,
      'a' => -10,
      'D' => 100,
      'S' => -100,
      'F' => 1000,
      'A' => -1000,
      _ => continue,
    };
    pos = std::cmp::max(0, pos);
    pos = std::cmp::min((states.len() - 1) as isize, pos);
  }

  Ok(())
}

fn play_manual() -> Result<(), std::io::Error> {
  Board::print_spacing();

  let mut board = Board(0);

  let mut fours = 0;
  fours += board.comp_move();
  fours += board.comp_move();

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
      _ => continue,
    }

    if new_board == board {
      continue;
    }

    board = new_board;
    fours += board.comp_move();
  }

  Ok(())
}

static mut SEED: u32 = 0x17004711;

fn rng(max: i32) -> i32 {
  let mut x = unsafe { SEED };
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  unsafe { SEED = x; }
  (x % (max as u32)) as i32
}

enum Command {
  AI { file: Option<String>, number: i32, until: i32 },
  Help(String, Option<String>),
  Manual,
  Replay(String),
}

fn parse_options(args: &[String]) -> Command
{
  let mut opts = Options::new();
  opts.optflag("h", "help", "Print this message.");
  opts.optopt("m", "max-tile", "Maximum tile value. Stop game once a tile with a value of 2^<number> has been reached.", "number");
  opts.optopt("n", "number", "Number of games to play. Defaults to 1", "number");
  opts.optopt("f", "file", "File to save replay in. If multiple games are played, a counter is added at the end of each file name.", "FILE");

  let brief = format!("Usage: {0} [options]\n       {0} replay FILE\n       {0} manual", args[0]);
  let options_str = opts.usage(&brief);

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(e) => { return Command::Help(options_str, Some(format!("{}", e))); }
  };

  if matches.free.first() == Some(&"replay".to_string()) &&
     matches.free.len() == 2 {
    return Command::Replay(matches.free[1].clone());
  } else if matches.free.first() == Some(&"manual".to_string()) &&
     matches.free.len() == 1 {
    return Command::Manual;
  } else if !matches.free.is_empty() {
    return Command::Help(options_str, Some(format!("Unknown argument: {}", matches.free[0])));
  }

  if matches.opt_present("h") {
    return Command::Help(options_str, None);
  }

  let max_tile = matches.opt_str("m").map_or(-1, |max_str|
    max_str.parse::<i32>().unwrap()
  );

  let file = matches.opt_str("f");

  let num_games = matches.opt_str("n").map_or(1, |num_str|
    num_str.parse::<i32>().unwrap()
  );

  Command::AI{ file, number: num_games, until: max_tile }
}

fn main() {
  init_score_table();

  let args: Vec<String> = std::env::args().collect();

  match parse_options(&args) {
    Command::Help(options_str, err) => {
      if let Some(err_str) = err {
        println!("{}", err_str);
      }
      println!("{}", options_str);
    }
    Command::Replay(file) => {
      replay(&file).unwrap();
    }
    Command::Manual => {
      play_manual().unwrap();
    }
    Command::AI{ file, number, until } => {
      let now = Instant::now();
      let mut tot_score = 0;
      for _ in 0..number {
        tot_score += ai_play(until, number == 1, file.as_ref()).unwrap();
      }
      let elapsed = now.elapsed();

      let time_sec = elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64) / 1_000_000_000f64;

      if number == 1 {
        println!("Time: {}", time_sec);  
      } else {
        println!("Average score: {}, time: {}", (tot_score as f32) / (number as f32), time_sec);
      }
    }
  }
}
