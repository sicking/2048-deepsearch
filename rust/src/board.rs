#![allow(dead_code)]

extern crate std;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Board(pub u64);

static mut PRINTED_LINES : usize = 0;

impl Board {
  pub fn print(self, fours: i32, backtrack: bool, extra: &str) {
    let nums = ["   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"];
    if backtrack && unsafe { PRINTED_LINES != 0 } {
      print!("\x1b[{}A", unsafe { PRINTED_LINES });
    }
    print!("+---+---+---+---+\n");
    for n in 0..16 {
      print!("|{}", nums[self.get_tile(15-n) as usize]);
      if (n % 4) == 3 {
        print!("|\n+---+---+---+---+\n");
      }
    }
    println!("Score: {}  ", self.game_score(fours));
    let mut lines = 10;
    if extra.len() != 0 {
      print!("{}", extra);
      if backtrack {
        lines += extra.lines().count();
      }
    }
    unsafe { PRINTED_LINES = if backtrack { lines } else { 0 } };
  }

  pub fn empty(self) -> i32 {
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

  pub fn distinct(self) -> u8 {
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

  pub fn max_val(self) -> i32 {
    let mut max = 0;
    for pos in 0..16 {
      max = std::cmp::max(max, self.get_tile(pos));
    }
    max
  }

  pub fn get_tile(self, tile: i32) -> i32 {
    debug_assert!(tile >= 0 && tile < 16);
    ((self.0 >> (tile * 4)) & 0xf) as i32
  }

  pub fn set_tile(self, tile: i32, val: i32) -> Board {
    debug_assert_eq!(self.get_tile(tile), 0);
    Board(self.0 | (val as u64) << (tile * 4))
  }

  pub fn comp_move(&mut self) -> i32 {
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

  pub fn slide(self, dir: i32) -> Board {
    match dir {
      0 => self.slide_right(),
      1 => self.slide_down(),
      2 => self.slide_left(),
      3 => self.slide_up(),
      _ => panic!("unknown direction"),
    }
  }

  pub fn slide_down(self) -> Board {
    let t = self.transpose();
    Board(unsafe {
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((t.0 >> 48) & 0xffff) as usize) as u64) << 48
      }).transpose()
  }

  pub fn slide_up(self) -> Board {
    let t = self.transpose();
    Board(unsafe {
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((t.0 >> 48) & 0xffff) as usize) as u64) << 48
      }).transpose()
  }

  pub fn slide_right(self) -> Board {
    Board(unsafe {
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_RIGHT_TABLE.get_unchecked(((self.0 >> 48) & 0xffff) as usize) as u64) << 48
      })
  }

  pub fn slide_left(self) -> Board {
    Board(unsafe {
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 0) & 0xffff) as usize) as u64) << 0 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 16) & 0xffff) as usize) as u64) << 16 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 32) & 0xffff) as usize) as u64) << 32 |
      (*SLIDE_LEFT_TABLE.get_unchecked(((self.0 >> 48) & 0xffff) as usize) as u64) << 48
      })
  }

  pub fn game_score(self, fours: i32) -> i32 {
    let mut score: i32 = 0;
    for pos in 0..16 {
      let val = ((self.0 >> (pos*4)) & 0xf) as i32;
      if val >= 2 {
        score += (val - 1) * (1 << val);
      }
    }
    score - (fours * 4)
  }

  pub fn transpose(self) -> Board {
    let a1 = self.0 & 0xf0f0_0f0f_f0f0_0f0f_u64;
    let a2 = self.0 & 0x0000_f0f0_0000_f0f0_u64;
    let a3 = self.0 & 0x0f0f_0000_0f0f_0000_u64;
    let a = a1 | (a2 << 12) | (a3 >> 12);
    let b1 = a      & 0xff00_ff00_00ff_00ff_u64;
    let b2 = a      & 0x00ff_00ff_0000_0000_u64;
    let b3 = a      & 0x0000_0000_ff00_ff00_u64;
    Board(b1 | (b2 >> 24) | (b3 << 24))
  }

  pub fn flip_horiz(self) -> Board {
    let a1 = self.0 & 0xffff_0000_ffff_0000_u64; // 1030
    let a2 = self.0 & 0x0000_ffff_0000_ffff_u64; // 0204
    let a = a1 | a2.rotate_right(32);            // 1432
    Board(a.rotate_left(16))                     // 4321
  }

  pub fn flip_vert(self) -> Board {
    let a1 = self.0 & 0xf000_f000_f000_f000_u64;
    let a2 = self.0 & 0x0f00_0f00_0f00_0f00_u64;
    let a3 = self.0 & 0x00f0_00f0_00f0_00f0_u64;
    let a4 = self.0 & 0x000f_000f_000f_000f_u64;
    Board(a1 >> 12 | a2 >> 4 | a3 << 4 | a4 << 12)
  }

  pub fn symmetries(self) -> BoardSymIter {
    BoardSymIter { op: 0, board: self }
  }
}

pub struct BoardSymIter {
  op: i32,
  board: Board,
}

impl Iterator for BoardSymIter {
  type Item = Board;
  fn next(&mut self) -> Option<Board> {
    // Might be simper to simply alternate calls to flip_horiz/transpose
    if self.op % 2 == 1 {
      self.board = self.board.flip_horiz();
    } else if self.op == 2 || self.op == 6 {
      self.board = self.board.flip_vert();
    } else if self.op == 4 {
      self.board = self.board.transpose();
    } else if self.op == 8 {
      return None;
    }
    self.op += 1;
    return Some(self.board);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sliding() {
    init();
    assert_eq!(Board(0x0001_0001_0002_0003).slide_up(),
               Board(0x0002_0002_0003_0000));
    assert_eq!(Board(0x0001_0001_0002_0003).slide_down(),
               Board(0x0000_0002_0002_0003));
    assert_eq!(Board(0x0001_0020_0300_1000).slide_down(),
               Board(0x0000_0000_0000_1321));
    assert_eq!(Board(0x0001_0020_0300_1001).slide_up(),
               Board(0x1322_0000_0000_0000));
    assert_eq!(Board(0x0001_0022_0300_1001).slide_up(),
               Board(0x1321_0002_0001_0000));
    assert_eq!(Board(0x1111_0000_0000_0000).slide_right(),
               Board(0x0022_0000_0000_0000));
    assert_eq!(Board(0x2235_0000_0000_0000).slide_left(),
               Board(0x3350_0000_0000_0000));
    assert_eq!(Board(0xcbbc_0010_0600_8000).slide_right(),
               Board(0x0ccc_0001_0006_0008));
    assert_eq!(Board(0x5550_0333_aa0a_c0cc).slide_right(),
               Board(0x0056_0034_00ab_00cd));
    assert_eq!(Board(0x1110_0222_bb0b_e0ee).slide_left(),
               Board(0x2100_3200_cb00_fe00));
    assert_eq!(Board(0x530c_50ac_53a0_03ac).slide_up(),
               Board(0x64bd_53ac_0000_0000));
    assert_eq!(Board(0x02be_10be_12b0_120e).slide_down(),
               Board(0x0000_0000_12be_23cf));
  }

  #[test]
  fn flipping() {
    init();
    assert_eq!(Board(0x0001_0a01_0002_0003).flip_vert(),
               Board(0x1000_10a0_2000_3000));
    assert_eq!(Board(0x1234_5678_9abc_def0).flip_vert(),
               Board(0x4321_8765_cba9_0fed));
    assert_eq!(Board(0x1234_5678_9abc_def0).flip_horiz(),
               Board(0xdef0_9abc_5678_1234));
    assert_eq!(Board(0x0101_2233_feef_ddee).flip_horiz(),
               Board(0xddee_feef_2233_0101));
    assert_eq!(Board(0x1234_5678_9abc_def0).transpose(),
               Board(0x159d_26ae_37bf_48c0));
    assert_eq!(Board(0x100a_02b0_0c30_d004).transpose(),
               Board(0x100d_02c0_0b30_a004));
  }

  #[test]
  fn iter() {
    init();
    let mut ans = vec![Board(0x1234_0000_0000_0000),
                       Board(0x1000_2000_3000_4000),
                       Board(0x4321_0000_0000_0000),
                       Board(0x0001_0002_0003_0004),
                       Board(0x0000_0000_0000_4321),
                       Board(0x0004_0003_0002_0001),
                       Board(0x0000_0000_0000_1234),
                       Board(0x4000_3000_2000_1000)];
    for board in Board(0x1234_0000_0000_0000).symmetries() {
      let pos = ans.iter().position(|x| *x == board);
      ans.remove(pos.unwrap());
    }
    assert!(ans.is_empty());
  }
}

static mut SLIDE_RIGHT_TABLE : [u16; 65536] = [0u16; 65536];
static mut SLIDE_LEFT_TABLE : [u16; 65536] = [0u16; 65536];

pub fn init() {
  for n in 0..65536 {
    let vals = [(n >> 0) & 0xf,
                (n >> 4) & 0xf,
                (n >> 8) & 0xf,
                (n >> 12) & 0xf];

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

static mut SEED: u32 = 0x17004711;
fn rng(max: i32) -> i32 {
  let mut x = unsafe { SEED };
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  unsafe { SEED = x; }
  (x % (max as u32)) as i32
}

