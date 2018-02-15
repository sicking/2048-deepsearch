package main

import (
  "board"
  "bufio"
  "os"
)

func main() {
  reader := bufio.NewReader(os.Stdin)
  b := board.Board(0).CompMove()
  for {
    b = b.CompMove()
    b.Print(true, "")

    hasValidMove := false
    for i := 0; i < 4; i++ {
      if b.Slide(i) != b {
        hasValidMove = true
        break
      }
    }

    if !hasValidMove {
      break
    }

    newboard := b
    for newboard == b {
      input, _ := reader.ReadString('\n')
      switch input {
      case "w\n":
        newboard = b.SlideUp()
      case "a\n":
        newboard = b.SlideLeft()
      case "s\n":
        newboard = b.SlideDown()
      case "d\n":
        newboard = b.SlideRight()
      }
    }

    b = newboard

  }
}
