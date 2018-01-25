#! python3

import tensorflow as tf
import numpy as np
import board as Board
import pdb

def board_to_array(board, arr, col):
  for i in range(16):
    arr[i, col] = (board & 0xf) / 13
    board >>= 4

def setup_network():
  n_x = 16
  n_y = 1
  n = [n_x, 30, 30, 10, n_y]
  alpha = 0.1

  X = tf.placeholder(tf.float32, shape=(n_x, None), name='X')
  Y = tf.placeholder(tf.float32, shape=(n_y, None), name='Y')
  prev_activ = X
  for l in range(1, len(n)):
    Wl = tf.get_variable("W" + str(l), [n[l], n[l-1]], dtype=tf.float32, initializer = tf.contrib.layers.xavier_initializer())
    bl = tf.get_variable("b" + str(l), [n[l], 1], dtype=tf.float32, initializer = tf.zeros_initializer())
    Zl = tf.matmul(Wl, prev_activ) + bl
    prev_activ = tf.nn.relu(Zl)

  cost = tf.nn.l2_loss(prev_activ-Y)
  optimizer = tf.train.AdamOptimizer(learning_rate=alpha).minimize(cost)

  return X, Y, prev_activ, optimizer


X_param, Y_param, yhat, optimizer = setup_network()

sess = tf.Session()
sess.run(tf.global_variables_initializer())
Board.print_spacing()

eval_arrays = [np.zeros((16, 1)),
               np.zeros((16, 2)),
               np.zeros((16, 3)),
               np.zeros((16, 4))]
prev_arr = np.zeros((16, 1))

while True:
  board = Board.comp_move(0)
  board_to_array(board, prev_arr, 0)

  while True:
    board = Board.comp_move(board)

    bestcol = None
    bestboard = None

    boards = []

    for dir in range(4):
      newboard = Board.slide(board, dir)
      if newboard != board:
        boards.append(newboard)

    if (len(boards)):
      arr = eval_arrays[len(boards) - 1]
      for i, newboard in enumerate(boards):
        board_to_array(newboard, arr, i)
      vals = sess.run(yhat, feed_dict = { X_param: arr })[0]
      bestval = float("-inf")
      for i, val in enumerate(vals):
        if val > bestval:
          bestval = val
          bestcol = i
          bestboard = newboard

    # learn
    prev_exp_val = bestval + 1 if bestboard else 0
    sess.run(optimizer, feed_dict = { X_param: prev_arr, Y_param: [[prev_exp_val]] })

    if bestboard == None:
      break

    np.copyto(prev_arr[:, 0], eval_arrays[len(boards) - 1][:, bestcol])
    board = bestboard

  Board.print(board)


#sess.close()
