import torch
import numpy as np
from api import Game
from utils import load
from model import Model

import sys

# NOTE: scripted model cannot be exported for unknown reason
checkpoint = load(sys.argv[1])
board_type = checkpoint['board_type']
dummy_game = Game(board_type)
model = Model(dummy_game.board_size)
model.load_state_dict(checkpoint['model_state_dict'])
r = checkpoint['r']

dummy_data = load('data_{:03}'.format(r))[0]
encoded_state = torch.tensor(np.expand_dims(dummy_data[0], 0), dtype=torch.float)

torch.onnx.export(
    model,
    (encoded_state, ),
    'exported_model.onnx',
    opset_version=17,
    verbose=True,
    input_names=["encoded_state"],
    output_names=["value"]
)

from onnxruntime.quantization import quantize_dynamic

# about 5% accuracy loss!
quantize_dynamic("exported_model.onnx", "quantized_model.onnx")
