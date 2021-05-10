import torch
import numpy as np
from api import Game, MCTS, set_random_seed
from utils import save, load
from model import Model, encode_input

import sys

# NOTE: scripted model cannot be exported to opset_7, which is the maximum version supported by onnx-js
checkpoint = load(sys.argv[1])
board_type = checkpoint['board_type']
dummy_game = Game(board_type)
model = Model(dummy_game.board_size, dummy_game.n_pieces)
model.load_state_dict(checkpoint['model_state_dict'])
r = checkpoint['r']

dummy_data = load('data_{:03}'.format(r))[0]
pieces = torch.from_numpy(np.expand_dims(dummy_data[0], 0))
mask = torch.from_numpy(np.expand_dims(dummy_data[1], 0))

torch.onnx.export(
    model,
    (pieces, mask),
    'exported_model.onnx',
    opset_version=7,
    verbose=True,
    input_names=["pieces", "mask"],
    output_names=["action_probs", "value"]
)
