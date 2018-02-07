#! python3

import tensorflow as tf
import numpy as np
import pdb
import math
import board as Board
import struct
from tensorflow.python import debug as tf_debug

def board_to_array(board, arr, col):
  for i in range(16):
    arr[i, col] = (board & 0xf) / 13
    board >>= 4

m_batch = 2048

def setup_network():
  n_x = 16
  n_y = 1
  n = [n_x, 100, 100, 100, 100, 100, 30, n_y]
  alpha = 0.01

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
    global_step = tf.get_variable("global_step", initializer=0, trainable=False)
    learning_rate = tf.train.exponential_decay(alpha, global_step,
                                               10000000/m_batch*10, 0.1, staircase=False)

    optimizer = tf.train.AdamOptimizer(learning_rate=learning_rate).minimize(cost, global_step=global_step)

  return X, Y, prev_activ, optimizer, cost, learning_rate


X_param, Y_param, yhat, optimizer, cost, learning_rate = setup_network()

sess = tf.Session()
#sess = tf_debug.LocalCLIDebugWrapperSession(sess)
sess.run(tf.global_variables_initializer())

def shuffle(X, Y):
  permutation = list(np.random.permutation(X.shape[1]))
  np.copyto(X, X[:, permutation])
  np.copyto(Y, Y[:, permutation])

eval_arrays = [np.zeros((16, 1)),
               np.zeros((16, 2)),
               np.zeros((16, 3)),
               np.zeros((16, 4))]

with open('../rust/2048training', 'rb') as f:
  m_train = struct.unpack('<L', f.read(4))[0]
  X_train = np.zeros((16, m_train))
  Y_train = np.zeros((1, m_train))
  for i in range(m_train):
    if (i % 1_000_000) == 0:
      print(f"reading data {i}")
    (board, exp_val) = struct.unpack('<Qf', f.read(12))
    board_to_array(board, X_train, i)
    Y_train[0, i] = exp_val


for epoch in range(100):
  print(f"Epoch: {epoch} ({sess.run([learning_rate])[0]})")
  shuffle(X_train, Y_train)

  n_batches = math.ceil(m_train/m_batch)
  total_cost = 0

  for batch_n in range(n_batches):
    X = X_train[:, (batch_n * m_batch):((batch_n+1) * m_batch)]
    Y = Y_train[:, (batch_n * m_batch):((batch_n+1) * m_batch)]

    batch_cost, _ = sess.run([cost, optimizer], feed_dict = { X_param: X, Y_param: Y })
    total_cost += batch_cost

  print(f"Cost {(total_cost / n_batches): 12,.0f}")

  for game_num in range(10):
    board = Board.comp_move(0)
    while True:
      board = Board.comp_move(board)
      bestboard = None
      newboards = []

      for dir in range(4):
        newboard = Board.slide(board, dir)
        if newboard != board:
          newboards.append(newboard)

      if len(newboards) == 0:
        print(f"Game {game_num + 1} score: {Board.game_score(board)}")
        break

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
          bestboard = newboards[j]

      board = bestboard


sess.close()
