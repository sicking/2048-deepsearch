#! python3

#import tensorflow as tf
#import numpy as np
import array


class Board:
  SLIDE_RIGHT_TABLE = array.array('H', range(65536))
  SLIDE_LEFT_TABLE = array.array('H', range(65536))

  def init():
    for n in range(65536):
      vals = [(n >> 0) & 0xf,
              (n >> 4) & 0xf,
              (n >> 8) & 0xf,
              (n >> 12) & 0xf]

      res = vals[0]
      merge_val = vals[0]
      dest_pos = -4 if merge_val == 0 else 0
      for pos in range(1, 4):
        val = vals[pos]
        if val == 0:
          pass
        elif val == merge_val:
          if (res >> dest_pos) & 0xf != 15:
            res += 1 << dest_pos
          merge_val = 0
        else:
          dest_pos += 4
          merge_val = val
          res |= val << dest_pos

      def reverse_row(row):
        return (((row & 0xf000) >> 12) |
                ((row & 0x0f00) >> 4) |
                ((row & 0x00f0) << 4) |
                ((row & 0x000f) << 12))

      Board.SLIDE_RIGHT_TABLE[n] = res
      Board.SLIDE_LEFT_TABLE[reverse_row(n)] = reverse_row(res)

  def print_spacing():
    print("\n\n\n\n\n\n\n\n\n\n\n\n\x1b[2A")

  def print(board):
    nums = ["   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"]
    print("\x1b[10A+---+---+---+---+")
    for n in range(16):
      print("|" + nums[Board.get_tile(board, 15-n)], end='')
      if ((n % 4) == 3):
        print("|\n+---+---+---+---+")
    print("Score: " + str(Board.game_score(board)) + "  ")

  def get_tile(board, tile):
    return (board >> (tile * 4)) & 0xf

  def set_tile(board, tile, val):
    return board | (val << (tile * 4))

  def empty(board):
    b = ~(board | (board >> 1) | (board >> 2) | (board >> 3)) & 0x1111_1111_1111_1111
    n1 = b + (b >> 16) + (b >> 32) + (b >> 48)
    n2 = (n1 + (n1 >> 4) + (n1 >> 8) + (n1 >> 12)) & 0xf
    return n2

  def comp_move(board):
    size = 16 if board == 0 else Board.empty(board)
    n = rng(size)
    pos = -1
    while n >= 0:
      pos += 1
      if Board.get_tile(board, pos) == 0:
        n -= 1
    four = rng(10) == 0
    return Board.set_tile(board, pos, 2 if four else 1)

  def slide(board, dir):
    if dir == 0:
      return Board.slide_right(board)
    if dir == 1:
      return Board.slide_down(board)
    if dir == 2:
      return Board.slide_left(board)
    if dir == 3:
      return Board.slide_up(board)

  def slide_down(board):
    t = Board.transpose(board);
    return Board.transpose(
      Board.SLIDE_RIGHT_TABLE[(t >> 0) & 0xffff] << 0 |
      Board.SLIDE_RIGHT_TABLE[(t >> 16) & 0xffff] << 16 |
      Board.SLIDE_RIGHT_TABLE[(t >> 32) & 0xffff] << 32 |
      Board.SLIDE_RIGHT_TABLE[(t >> 48) & 0xffff] << 48)

  def slide_up(board):
    t = Board.transpose(board);
    return Board.transpose(
      Board.SLIDE_LEFT_TABLE[(t >> 0) & 0xffff] << 0 |
      Board.SLIDE_LEFT_TABLE[(t >> 16) & 0xffff] << 16 |
      Board.SLIDE_LEFT_TABLE[(t >> 32) & 0xffff] << 32 |
      Board.SLIDE_LEFT_TABLE[(t >> 48) & 0xffff] << 48)

  def slide_right(board):
    return (
      Board.SLIDE_RIGHT_TABLE[(board >> 0) & 0xffff] << 0 |
      Board.SLIDE_RIGHT_TABLE[(board >> 16) & 0xffff] << 16 |
      Board.SLIDE_RIGHT_TABLE[(board >> 32) & 0xffff] << 32 |
      Board.SLIDE_RIGHT_TABLE[(board >> 48) & 0xffff] << 48)

  def slide_left(board):
    return (
      Board.SLIDE_LEFT_TABLE[(board >> 0) & 0xffff] << 0 |
      Board.SLIDE_LEFT_TABLE[(board >> 16) & 0xffff] << 16 |
      Board.SLIDE_LEFT_TABLE[(board >> 32) & 0xffff] << 32 |
      Board.SLIDE_LEFT_TABLE[(board >> 48) & 0xffff] << 48)

  def game_score(board):
    score = 0
    for pos in range(16):
      val = Board.get_tile(board, pos)
      if val >= 2:
        score += (val - 1) * (1 << val)
    return score

  def transpose(board):
    a1 = board & 0xf0f0_0f0f_f0f0_0f0f
    a2 = board & 0x0000_f0f0_0000_f0f0
    a3 = board & 0x0f0f_0000_0f0f_0000
    a = a1 | (a2 << 12) | (a3 >> 12)
    b1 = a     & 0xff00_ff00_00ff_00ff
    b2 = a     & 0x00ff_00ff_0000_0000
    b3 = a     & 0x0000_0000_ff00_ff00
    return b1 | (b2 >> 24) | (b3 << 24)


SEED = 0x17004711
def rng(max):
  global SEED
  x = SEED
  x ^= (x << 13) & 0xffff_ffff
  x ^= x >> 17
  x ^= (x << 5) & 0xffff_ffff
  SEED = x
  return x % max



Board.init()
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
