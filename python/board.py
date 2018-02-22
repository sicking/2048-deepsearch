import array
import builtins

_SLIDE_RIGHT_TABLE = array.array('H', range(65536))
_SLIDE_LEFT_TABLE = array.array('H', range(65536))

def _init():
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

    _SLIDE_RIGHT_TABLE[n] = res
    _SLIDE_LEFT_TABLE[reverse_row(n)] = reverse_row(res)

def print_spacing():
  builtins.print("\n\n\n\n\n\n\n\n\n\n\n\n\x1b[2A")

def print(board, overwrite=True):
  nums = ["   ", "  2", "  4", "  8", " 16", " 32", " 64", "128", "256", "512", " 1K", " 2K", " 4K", " 8K", "16K", "32K"]
  if overwrite:
    builtins.print("\x1b[10A", end='')
  builtins.print("+---+---+---+---+")
  for n in range(16):
    builtins.print("|" + nums[get_tile(board, 15-n)], end='')
    if ((n % 4) == 3):
      builtins.print("|\n+---+---+---+---+")
  builtins.print("Score: " + str(game_score(board)) + "  ")

def get_tile(board, tile):
  return (board >> (tile * 4)) & 0xf

def set_tile(board, tile, val):
  return board | (val << (tile * 4))

def empty(board):
  b = ~(board | (board >> 1) | (board >> 2) | (board >> 3)) & 0x1111_1111_1111_1111
  n1 = (b + (b >> 4) + (b >> 8) + (b >> 12)) & 0x000f_000f_000f_000f
  n2 = (n1 + (n1 >> 16) + (n1 >> 32) + (n1 >> 48)) & 0x1f
  return n2

def comp_move(board):
  n = _rng(empty(board))
  pos = -1
  while n >= 0:
    pos += 1
    if get_tile(board, pos) == 0:
      n -= 1
  four = _rng(10) == 0
  return set_tile(board, pos, 2 if four else 1)

def slide(board, dir):
  if dir == 0:
    return slide_right(board)
  if dir == 1:
    return slide_down(board)
  if dir == 2:
    return slide_left(board)
  if dir == 3:
    return slide_up(board)

def slide_down(board):
  t = transpose(board);
  return transpose(
    _SLIDE_RIGHT_TABLE[(t >> 0) & 0xffff] << 0 |
    _SLIDE_RIGHT_TABLE[(t >> 16) & 0xffff] << 16 |
    _SLIDE_RIGHT_TABLE[(t >> 32) & 0xffff] << 32 |
    _SLIDE_RIGHT_TABLE[(t >> 48) & 0xffff] << 48)

def slide_up(board):
  t = transpose(board);
  return transpose(
    _SLIDE_LEFT_TABLE[(t >> 0) & 0xffff] << 0 |
    _SLIDE_LEFT_TABLE[(t >> 16) & 0xffff] << 16 |
    _SLIDE_LEFT_TABLE[(t >> 32) & 0xffff] << 32 |
    _SLIDE_LEFT_TABLE[(t >> 48) & 0xffff] << 48)

def slide_right(board):
  return (
    _SLIDE_RIGHT_TABLE[(board >> 0) & 0xffff] << 0 |
    _SLIDE_RIGHT_TABLE[(board >> 16) & 0xffff] << 16 |
    _SLIDE_RIGHT_TABLE[(board >> 32) & 0xffff] << 32 |
    _SLIDE_RIGHT_TABLE[(board >> 48) & 0xffff] << 48)

def slide_left(board):
  return (
    _SLIDE_LEFT_TABLE[(board >> 0) & 0xffff] << 0 |
    _SLIDE_LEFT_TABLE[(board >> 16) & 0xffff] << 16 |
    _SLIDE_LEFT_TABLE[(board >> 32) & 0xffff] << 32 |
    _SLIDE_LEFT_TABLE[(board >> 48) & 0xffff] << 48)

def game_score(board):
  score = 0
  for pos in range(16):
    val = get_tile(board, pos)
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


_SEED = 0x17004711
def _rng(max):
  global _SEED
  x = _SEED
  x ^= (x << 13) & 0xffff_ffff
  x ^= x >> 17
  x ^= (x << 5) & 0xffff_ffff
  _SEED = x
  return x % max

_init()