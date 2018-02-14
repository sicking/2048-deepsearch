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
  return 0
}

func (board Board) GetTile(tile int) int {
  return int((uint64(board) >> (uint(tile) * 4)) & 0xf)
}
