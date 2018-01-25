#! python3

import tensorflow as tf
import numpy as np
import board as Board
import pdb

def board_to_array(board):
  res = []
  for _ in range(16):
    res.append([(board & 0xf) / 13])
    board >>= 4
  return res

def setup_network():
  n_x = 16
  m = 1
  n = [n_x, 30, 30, 20, 10, 1]
  alpha = 0.005

  X = tf.placeholder(tf.float32, shape=(n_x, m), name='X')
  Y = tf.placeholder(tf.float32, shape=(n[-1], m), name='Y')
  vars = {}
  prev_activ = X
  for l in range(1, len(n)):
    vars["W" + str(l)] = tf.get_variable("W" + str(l), [n[l], n[l-1]], initializer = tf.contrib.layers.xavier_initializer())
    vars["b" + str(l)] = tf.get_variable("b" + str(l), [n[l], 1], initializer = tf.zeros_initializer())
    Zl = tf.matmul(vars["W" + str(l)], prev_activ) + vars["b" + str(l)]
    prev_activ = tf.nn.relu(Zl)

  cost = tf.nn.l2_loss(prev_activ-Y)
  optimizer = tf.train.AdamOptimizer(learning_rate=alpha).minimize(cost)

  return vars, X, Y, prev_activ, optimizer


variables, X_param, Y_param, yhat, optimizer = setup_network()

sess = tf.Session()
sess.run(tf.global_variables_initializer())
Board.print_spacing()


while True:
  board = Board.comp_move(0)
  prev_arr = board_to_array(board)

  while True:
    board = Board.comp_move(board)
    #Board.print(board, overwrite=False)

    bestval = float("-inf")
    bestarr = None
    bestboard = None
    bestdir = None

    for dir in range(4):
      newboard = Board.slide(board, dir)
      if newboard == board:
        continue

      arr = board_to_array(newboard)
      val = sess.run(yhat, feed_dict = { X_param: arr })[0][0]
      if val > bestval:
        bestval = val
        bestarr = arr
        bestboard = newboard
        bestdir = dir

    #print(f"bestdir: {bestdir}, bestval: {bestval}")

    # learn
    prev_exp_val = bestval + 1 if bestarr else 0
    sess.run(optimizer, feed_dict = { X_param: prev_arr, Y_param: [[prev_exp_val]] })

    if bestboard == None:
      #Board.print(board, overwrite=False)
      #pdb.set_trace()
      break

    prev_arr = bestarr
    board = bestboard

  Board.print(board)


sess.close()