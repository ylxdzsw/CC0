# proof of concept: predict the possible moves

import torch
import numpy as np
from utils import save, load, normalize
from environment import Environment

try:
    load("data")
except:
    data = []

    while len(data) < 5000:
        env = Environment()
        for i in range(200):
            env.random_move()
            if i > 50: # only starts from there
                possible_moves = env.get_possible_moves()
                p = normalize([len(moves) for pos, moves in possible_moves]) # prefer pieces that have more possible moves
                i = np.random.choice(range(len(possible_moves)), p=p)
                self_pieces, oppo_pieces = env.dump()
                data.append((self_pieces, oppo_pieces, possible_moves[i][0], possible_moves[i][1]))

    save(data, "data")

