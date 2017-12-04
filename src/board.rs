#![allow(dead_code)]

extern crate std;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Board(pub u64);

impl Board {
  pub fn print_spacing() {
    print!("\n\n\n\n\n\n\n\n\n\n\n\n\x1b[2A");
  }
  pub fn print(self, fours: i32) {
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

