import torch
import numpy as np
from api import Game, MCTS
from utils import save, load
from model import Model, encode_input

# playout a game and return [(state, action_probs, value)] of each selected node
def self_play(board_type, model):
    def policy_fun(game):
        model.eval()
        with torch.no_grad():
            pieces, mask = encode_input(game)
            pieces = torch.as_tensor(np.expand_dims(pieces, 0))
            mask = torch.as_tensor(np.expand_dims(mask, 0))
            policy, value = model(pieces, mask)
            return torch.squeeze(policy, 0), torch.squeeze(value, 0)

    game = Game(board_type)
    mcts = MCTS(policy_fun)

    while True:
        status = game.get_status()
        if status != 0:
            break

        mcts.playout(game, 800 - mcts.total_visits())
        old_pos, new_pos = mcts.sample_action(0., 1e-3)

        print(old_pos, ',', new_pos)

        game.do_move(old_pos, new_pos)
        mcts.chroot(old_pos, new_pos)


def collect_self_play_data():
    # multi process
    pass

def rotate_and_flip():
    pass

def train_step():
    pass

def evaluate():
    pass

model = Model(121)

self_play("standard", model)
