import torch
import numpy as np
from api import Game, MCTS, set_random_seed
from utils import save, load
from model import Model, encode_input

import sys

checkpoint = load(sys.argv[1])
model = torch.jit.script(Model(121, 10))
model.load_state_dict(checkpoint['model_state_dict'])
model.cpu().eval()
@torch.no_grad()
def policy_fun(game):
    pieces, mask = encode_input(game)
    pieces = torch.as_tensor(np.expand_dims(pieces, 0))
    mask = torch.as_tensor(np.expand_dims(mask, 0))
    policy, value = model(pieces, mask)
    return torch.squeeze(policy, 0), torch.squeeze(value, 0)

game = Game("standard")
mcts = MCTS(policy_fun)

while True:
    status = game.get_status()
    if status != 0:
        if status == 1:
            print("player 1 won")
        if status == 2:
            print("player 2 won")
        if status == 3:
            print("tie")
        break

    mcts.playout(game, 8000 - mcts.total_visits())
    value = mcts.root_value()
    old_pos, new_pos = mcts.sample_action(0, 0.1)

    print("move from {} to {}. value: {}".format(old_pos, new_pos, value))

    game.do_move(old_pos, new_pos)
    mcts.chroot(old_pos, new_pos)
