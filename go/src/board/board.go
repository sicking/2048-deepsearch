package board

import (
  "fmt"
  "strings"
)

type Board uint64

var (
  printed_lines int = 0
)

func (board Board) Print(backtrack bool, extra string) {
  nums := [...]string {"   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"}
  if backtrack && printed_lines != 0 {
    fmt.Printf("\x1b[%dA", printed_lines)
  }
  fmt.Print("+---+---+---+---+\n")
  for n := 0; n < 16; n++ {
    fmt.Print("|", nums[board.GetTile(15-n)])
    if (n % 4) == 3 {
      fmt.Print("|\n+---+---+---+---+\n")
    }
  }
  fmt.Printf("Score: %d  \n", board.GameScore())
  fmt.Print(extra)

  if backtrack {
    printed_lines = 10 + strings.Count(extra, "\n")
  } else {
    printed_lines = 0
  }
}

func (board Board) GameScore() int {
  score := 0
  for pos := 0; pos < 16; pos++ {
    val := board.GetTile(pos)
    if val >= 2 {
      score += (val - 1) * (1 << uint(val))
    }
  }
  return score
}

func (board Board) GetTile(tile int) int {
  return int((uint64(board) >> (uint(tile) * 4)) & 0xf)
}

func (board Board) SetTile(tile int, val int) Board {
  return board | Board((val << (uint(tile) * 4)))
}

func (board Board) Empty() uint {
  b := ^(board | (board >> 1) | (board >> 2) | (board >> 3)) & 0x1111111111111111
  n1 := (b + (b >> 4) + (b >> 8) + (b >> 12)) & 0x000f000f000f000f
  n2 := uint(n1 + (n1 >> 16) + (n1 >> 32) + (n1 >> 48)) & 0x1f
  return n2
}

func (board Board) CompMove() Board {
  n := int32(rng(uint32(board.Empty())))
  pos := -1
  for n >= 0 {
    pos += 1
    if board.GetTile(pos) == 0 {
      n -= 1
    }
  }
  val := 1
  if rng(10) == 0 {
    val = 2
  }
  return board.SetTile(pos, val)
}

var seed uint32 = 0x17004711;
func rng(max uint32) uint32 {
  x := seed;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  seed = x
  return x % max
}

func (board Board) Slide(dir int) Board {
  switch dir {
  case 0:
    return board.SlideRight()
  case 1:
    return board.SlideDown()
  case 2:
    return board.SlideLeft()
  case 3:
    return board.SlideUp()
  }
  panic("Unknown slide direction")
}

func (board Board) SlideDown() Board {
  t := board.Transpose()
  return Board(uint64(slide_right_table[(t >> 0) & 0xffff]) << 0 |
               uint64(slide_right_table[(t >> 16) & 0xffff]) << 16 |
               uint64(slide_right_table[(t >> 32) & 0xffff]) << 32 |
               uint64(slide_right_table[(t >> 48) & 0xffff]) << 48).Transpose()
}

func (board Board) SlideUp() Board {
  t := board.Transpose()
  return Board(uint64(slide_left_table[(t >> 0) & 0xffff]) << 0 |
               uint64(slide_left_table[(t >> 16) & 0xffff]) << 16 |
               uint64(slide_left_table[(t >> 32) & 0xffff]) << 32 |
               uint64(slide_left_table[(t >> 48) & 0xffff]) << 48).Transpose()
}

func (board Board) SlideRight() Board {
  return Board(uint64(slide_right_table[(board >> 0) & 0xffff]) << 0 |
               uint64(slide_right_table[(board >> 16) & 0xffff]) << 16 |
               uint64(slide_right_table[(board >> 32) & 0xffff]) << 32 |
               uint64(slide_right_table[(board >> 48) & 0xffff]) << 48)
}

func (board Board) SlideLeft() Board {
  return Board(uint64(slide_left_table[(board >> 0) & 0xffff]) << 0 |
               uint64(slide_left_table[(board >> 16) & 0xffff]) << 16 |
               uint64(slide_left_table[(board >> 32) & 0xffff]) << 32 |
               uint64(slide_left_table[(board >> 48) & 0xffff]) << 48)
}

func (board Board) Transpose() Board {
  a1 := board & 0xf0f00f0ff0f00f0f
  a2 := board & 0x0000f0f00000f0f0
  a3 := board & 0x0f0f00000f0f0000
  a := a1 | (a2 << 12) | (a3 >> 12)
  b1 := a     & 0xff00ff0000ff00ff
  b2 := a     & 0x00ff00ff00000000
  b3 := a     & 0x00000000ff00ff00
  return b1 | (b2 >> 24) | (b3 << 24)
}

var (
  slide_right_table [65536]uint16
  slide_left_table [65536]uint16
)

func reverse_row(row uint16) uint16 {
  return (((row & 0xf000) >> 12) |
          ((row & 0x0f00) >> 4) |
          ((row & 0x00f0) << 4) |
          ((row & 0x000f) << 12))
}

func init() {
  for n := 0; n < 65536; n++ {
    vals := [4]uint16 {
              uint16((n >> 0) & 0xf),
              uint16((n >> 4) & 0xf),
              uint16((n >> 8) & 0xf),
              uint16((n >> 12) & 0xf),
            }

    res := vals[0]
    merge_val := vals[0]
    dest_pos := uint(0)
    if merge_val == 0 {
      dest_pos -= 4
    }

    for pos := 1; pos < 4; pos++ {
      val := vals[pos]
      if val == 0 {
        // Do nothing
      } else if val == merge_val {
        if (res >> dest_pos) & 0xf != 15 {
          res += 1 << dest_pos
        }
        merge_val = 0
      } else {
        dest_pos += 4
        merge_val = val
        res |= val << dest_pos
      }
    }

    slide_right_table[n] = res
    slide_left_table[reverse_row(uint16(n))] = reverse_row(res)
  }
}
