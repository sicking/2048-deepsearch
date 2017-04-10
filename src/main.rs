extern crate rand;
extern crate getch;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Board(u64);

impl Board {
  fn print_spacing() {
    for _ in 0..12 { println!(); }
  }
  fn print(self, fours: i32) {
    let nums = ["   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"];
    print!("\x1b[10A+---+---+---+---+\n");
    for n in 0..16 {
      let v = (self.0 >> ((15-n) * 4)) & 0xf;
      print!("|{}", nums[v as usize]);
      if (n % 4) == 3 {
        print!("|\n+---+---+---+---+\n");
      }
    }
    println!("Score: {}      ", self.game_score(fours));
  }

  fn empty(self) -> i32 {
    #[cfg(debug_assertions)]
    fn empty_debug(num: u64) -> i32 {
      let mut empty = 0;
      for i in 0..16 {
        if (num >> (i * 4)) & 0xf == 0 {
          empty += 1;
        }
      }
      empty
    }

    // Below can't handle this case.
    assert_ne!(self.0, 0);

    let b = !(self.0 | (self.0 >> 1) | (self.0 >> 2) | (self.0 >> 3)) & 0x1111_1111_1111_1111u64;
    let n1 = b + (b >> 16) + (b >> 32) + (b >> 48);
    let n2 = (n1 + (n1 >> 4) + (n1 >> 8) + (n1 >> 12)) & 0xf;
    assert!((n2 as i32) == empty_debug(self.0) || n2 == 0);
    n2 as i32
  }

  fn comp_move(&mut self) -> i32 {
    // self.empty() can't handle the completely-empty case.
    assert!(self.0 == 0 || self.empty() > 0);
    let size = if self.0 == 0 { 16 } else { self.empty() };
    let mut n = (rand::random::<f32>() * (size as f32)).floor() as i32;
    assert!(self.0 == 0 || n < self.empty());
    let mut pos = -1;
    while n >= 0 {
      pos += 1;
      if (self.0 >> (pos * 4)) & 0xf == 0 {
        n -= 1;
      }
    }
    let four = rand::random::<f32>() < 0.1;
    self.0 |= if four { 2 } else { 1 } << (pos * 4);
    if four { 1 } else { 0 }
  }

  fn slide_down(&mut self) {
    self.do_slide(0, 4, 16);
  }

  fn slide_up(&mut self) {
    self.do_slide(16*3, 4, -16);
  }

  fn slide_right(&mut self) {
    self.do_slide(0, 16, 4);
  }

  fn slide_left(&mut self) {
    self.do_slide(4*3, 16, -4);
  }

  fn do_slide(&mut self, start: i32, sec_move: i32, step_move: i32) {
    let mut sec = start;
    for _ in 0..4 {
      let mut merge_val = (self.0 >> sec) & 0xf;
      let mut dest_pos = if merge_val == 0 { sec - step_move } else { sec };
      let mut pos = sec;
      for _ in 1..4 {
        pos += step_move;
        let val = (self.0 >> pos) & 0xf;

        if val == 0 {
          // do nothing
        } else if val == merge_val {
          self.0 += 1 << dest_pos;
          self.0 &= !(0xf << pos);
          merge_val = 0;
        } else {
          dest_pos += step_move;
          merge_val = val;
          self.0 &= !(0xf << pos);
          self.0 |= val << dest_pos;
        }
      }

      sec += sec_move;
    }
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

    let a1 = self.0 & 0xf0f0_0f0f_f0f0_0f0f_u64;
    let a2 = self.0 & 0x0000_f0f0_0000_f0f0_u64;
    let a3 = self.0 & 0x0f0f_0000_0f0f_0000_u64;
    let a = a1 | (a2 << 12) | (a3 >> 12);
    let b1 = a & 0xff00_ff00_00ff_00ff_u64;
    let b2 = a & 0x00ff_00ff_0000_0000_u64;
    let b3 = a & 0x0000_0000_ff00_ff00_u64;
    Board(b1 | (b2 >> 24) | (b3 << 24))
  }

}

fn main() {
  Board::print_spacing();

  play_manual().unwrap();
}

fn play_manual() -> std::result::Result<(), std::io::Error> {
  let mut board = Board(0);

  let mut fours = 0;
  fours += board.comp_move();
  fours += board.comp_move();

  let io = getch::Getch::new()?;

  loop {
    board.print(fours);

    let mut new_board = board;
    match io.getch()? as char {
      'q' => break,
      'w' => new_board.slide_up(),
      's' => new_board.slide_down(),
      'd' => new_board.slide_right(),
      'a' => new_board.slide_left(),
      't' => { board = new_board.transpose(); continue },
      _ => (),
    }

    if new_board == board {
      continue;
    }

    board = new_board;
    fours += board.comp_move();
  }

  Ok(())
}
