import torch
import numpy as np
from utils import save, load
from model import Model, encode_input

# NOTE: scripted model cannot be exported to opset_7, which is the maximum version supported by onnx-js
model = Model(121, 10)
checkpoint = load(sys.argv[1])
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
    output_names=["action_probs"]
)
