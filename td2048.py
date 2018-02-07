#! python3

import tensorflow as tf
import numpy as np
import board as Board
import pdb
import math
from tensorflow.python import debug as tf_debug

def board_to_array(board, arr, col):
  for i in range(16):
    arr[i, col] = (board & 0xf) / 13
    board >>= 4

m = 256
alpha = 0.001
def setup_network():
  n_x = 16
  n_y = 1
  n = [n_x, 500, 500, 100, 100, 100, 100, n_y]

  with tf.name_scope("inputs"):
    X = tf.placeholder(tf.float32, shape=(n_x, None), name='X')
    Y = tf.placeholder(tf.float32, shape=(n_y, None), name='Y')

  prev_activ = X
  for l in range(1, len(n)):
    with tf.name_scope("layer_" + str(l)):
      with tf.name_scope("weights"):
        Wl = tf.get_variable("W" + str(l), [n[l], n[l-1]], dtype=tf.float32, initializer = tf.contrib.layers.xavier_initializer(seed=l))
      with tf.name_scope("biases"):
        bl = tf.get_variable("b" + str(l), [n[l], 1], dtype=tf.float32, initializer = tf.zeros_initializer())
      with tf.name_scope("linear"):
        Zl = tf.matmul(Wl, prev_activ) + bl
      with tf.name_scope("activation"):
        prev_activ = tf.nn.relu(Zl)

  with tf.name_scope("cost"):
    cost = tf.nn.l2_loss(prev_activ-Y)
  with tf.name_scope("optim"):
    alpha = tf.placeholder(tf.float32, name='alpha')
    optimizer = tf.train.AdamOptimizer(learning_rate=alpha).minimize(cost)

  return X, Y, prev_activ, optimizer, cost, alpha


X_param, Y_param, yhat, optimizer, cost, alpha_param = setup_network()

sess = tf.Session()
#sess = tf_debug.LocalCLIDebugWrapperSession(sess)
sess.run(tf.global_variables_initializer())

eval_arrays = [np.zeros((16, 1)),
               np.zeros((16, 2)),
               np.zeros((16, 3)),
               np.zeros((16, 4))]

prev_arr = np.zeros((16, m))
prev_arr_next = np.zeros((16, m))
boards = np.zeros(m, np.uint64)
prev_exp_vals = np.zeros((1, m))

for i in range(m):
  boards[i] = Board.comp_move(0)
  board_to_array(int(boards[i]), prev_arr, i)

while True:
  for i in range(m):
    board = Board.comp_move(int(boards[i]))

    bestcol = None
    bestboard = None

    newboards = []

    for dir in range(4):
      newboard = Board.slide(board, dir)
      if newboard != board:
        newboards.append(newboard)

    if (len(newboards)):
      arr = eval_arrays[len(newboards) - 1]
      for j, newboard in enumerate(newboards):
        board_to_array(newboard, arr, j)
      vals = sess.run(yhat, feed_dict = { X_param: arr })[0]
      bestval = float("-inf")
      for j, val in enumerate(vals):
        if math.isnan(val):
          print("Found NaN!")
          quit()

        if val > bestval:
          bestval = val
          bestcol = j
          bestboard = newboards[j]

    # learn
    prev_exp_vals[0][i] = bestval + 1 if bestboard else 0

    if bestboard == None:
      #Board.print(board)
      print("Game score: " + str(Board.game_score(board)))
      bestboard = Board.comp_move(0)
      newboards.append(bestboard)
      bestcol = 0
      board_to_array(bestboard, eval_arrays[0], 0)

    np.copyto(prev_arr_next[:, i], eval_arrays[len(newboards) - 1][:, bestcol])
    boards[i] = bestboard


  cost_pre, _ = sess.run([cost, optimizer], feed_dict = { X_param: prev_arr, Y_param: prev_exp_vals, alpha_param: alpha })
  cost_post = sess.run(cost, feed_dict = { X_param: prev_arr, Y_param: prev_exp_vals })

  print(f"Cost pre: {math.sqrt(cost_pre*2/m):7.2f}, imp: {(cost_pre-cost_post)/cost_pre*100:5.2f}%, alpha: {alpha:.10f}")
  if (cost_pre-cost_post)/cost_pre < 0:
    alpha *= 0.9
  elif (cost_pre-cost_post)/cost_pre < 0.02:
    alpha *= 1.05
  prev_arr, prev_arr_next = prev_arr_next, prev_arr


#sess.close()
