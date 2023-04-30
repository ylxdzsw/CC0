import torch
import numpy as np
from api import Game
from utils import load
from model import Model

import sys

checkpoint = load(sys.argv[1])
board_type = checkpoint['board_type']
dummy_game = Game(board_type)
model = Model(dummy_game.board_size)
model.load_state_dict(checkpoint['model_state_dict'])
r = checkpoint['r']

data = load('data_{:03}'.format(r))[:256]

for encoded_state, y in data:
    x = np.array(encoded_state)
    x = np.expand_dims(x, 0)
    p = model(torch.tensor(x, dtype=torch.float))
    print(p.item(), y)
