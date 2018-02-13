mod board;

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
use futures::Future;
use futures_cpupool::CpuPool;
use std::cell::RefCell;
use board::Board;

thread_local!(static HASH: RefCell<HashMap<Board, (i32, f32, f32)>> = RefCell::new(HashMap::new()));

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

  let mut state = PlayState::ZeroProbDeath;

  loop {
    let mut bestdir = -1;
    let mut searched_depth = 0;
    let mut bestexp = 0f32;
    let mut best_end_prob = 1f32;
    let mut depth: u8;

    let mut searches = 0;

    while {
      depth = match state {
        PlayState::ZeroProbDeath => std::cmp::max(3, std::cmp::max(board.distinct(), 4) - 4),
        PlayState::LowProbDeath => std::cmp::max(3, std::cmp::max(board.distinct(), 2) - 2),
        PlayState::HighPropDeath => std::cmp::max(3, board.distinct()),
        PlayState::VeryHighProbDeath => 17,
      };
      depth > searched_depth } {

      bestdir = -1;
      bestexp = 0.0;
      best_end_prob = 1.0;

      let res = futures::future::join_all((0..4).map(|dir| {
        pool.spawn_fn(move || -> Result<(f32, f32), ()> {
          let new_board = board.slide(dir);
          if new_board == board {
            Ok((-1.0f32, 1.0f32))
          } else {
            HASH.with(|hash_cell| {
              let mut hash = hash_cell.borrow_mut();
              hash.clear();
              Ok(ai_comp_move(new_board, depth, &mut *hash, 1f32))
            })
          }
        })
      })).wait().unwrap();

      for (dir, &(exp, end_prob)) in res.iter().enumerate() {
        if exp > bestexp {
          bestexp = exp;
          bestdir = dir as i32;
          best_end_prob = end_prob;
        }
      }

      searched_depth = depth;
      searches += 1;

      state = PlayState::from_prob(best_end_prob);
    }

    if print {
      board.print(fours, true,
                  &format!("Death prob: {:.9}\nDepth: {} State: {:?}          \n", best_end_prob, depth, state));
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
  }

  Ok(board.game_score(fours))
}

fn ai_comp_move(board: Board, depth: u8, hash: &mut HashMap<Board, (i32, f32, f32)>, prob: f32) -> (f32, f32) {
  if depth <= 0 || prob < 0.0001 {
    return (board.heur_score(), 0f32);
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

impl Board {
  fn heur_score(self) -> f32 {
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
  }

  board::init();
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

  loop {
    let state = &states[pos as usize];

    state.board.print(state.fours, true,
                      &format!("Move: {}      \n\
                                Expected heuristic score: {:.2}      \n\
                                Probability of death: {:.9} ({:?})       \n\
                                Searched depth: {}  \n\
                                Number of searches: {}  \n\
                                {}\n",
                                pos,
                                state.bestexp,
                                state.best_end_prob, PlayState::from_prob(state.best_end_prob),
                                state.depth,
                                state.searches,
                                if state.bestdir == -1 {
                                  format!("End of game.         ")
                                } else {
                                  assert!(state.bestdir <= 3);
                                  format!("Decided direction: {}", "RDLU".chars().nth(state.bestdir as usize).unwrap())
                                }
                                ));

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
  let mut board = Board(0);

  let mut fours = 0;
  fours += board.comp_move();
  fours += board.comp_move();

  let io = getch::Getch::new()?;

  loop {
    board.print(fours, true, "");

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
