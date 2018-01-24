#! python3

#import tensorflow as tf
#import numpy as np
import board as Board

def _find_getch():
    try:
        import termios
    except ImportError:
        # Non-POSIX. Return msvcrt's (Windows') getch.
        import msvcrt
        return msvcrt.getch

    # POSIX system. Create and return a getch that manipulates the tty.
    import sys, tty
    def _getch():
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            tty.setraw(fd)
            ch = sys.stdin.read(1)
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        return ch

    return _getch
getch = _find_getch()


Board.print_spacing()

board = 0
board = Board.comp_move(board)
board = Board.comp_move(board)

while True:
  Board.print(board)
  newboard = board
  while newboard == board:
    c = getch()
    if c == 'd':
      newboard = Board.slide(board, 0)
    if c == 's':
      newboard = Board.slide(board, 1)
    if c == 'a':
      newboard = Board.slide(board, 2)
    if c == 'w':
      newboard = Board.slide(board, 3)
    if c == 'q':
      quit()
  board = Board.comp_move(newboard)
