import torch
import numpy as np
from multiprocessing import Pool
from api import Game, MCTS
from utils import save, load
from model import Model, encode_input

# playout a game and collect game trace ([(self_pieces, oppo_pieces, action_probs)], result)
def self_play(board_type, model):
    @torch.no_grad()
    def policy_fun(game):
        pieces, mask = encode_input(game)
        pieces = torch.as_tensor(np.expand_dims(pieces, 0))
        mask = torch.as_tensor(np.expand_dims(mask, 0))
        policy, value = model(pieces, mask)
        return torch.squeeze(policy, 0), torch.squeeze(value, 0)

    game = Game(board_type)
    mcts = MCTS(policy_fun)

    record = []

    while True:
        status = game.get_status()
        if status != 0:
            break

        mcts.playout(game, 800 - mcts.total_visits())

        state = game.dump()
        action_probs = mcts.get_action_probs(1e-3)
        value = mcts.root_value()

        record.append((*state, action_probs))
        print(state, action_probs, value)

        old_pos, new_pos = mcts.sample_action(0., 1e-3)
        game.do_move(old_pos, new_pos)
        mcts.chroot(old_pos, new_pos)

    return record

# replay the game and generate masks for training. Also augment the data by rotating and fliping.
def trace_to_data():
    pass

# load the model and self-play several rounds. This method is used as the entry point of workers.
def self_play_batch(board_type, model_path, ntimes):
    pass

# inference performance: CUDA: 60 playouts/s, SingleThreadCPU: 80 playouts/s, MultiThreadCPU: 105 playouts/s but uses 40% of all 16 cores.
# TorchScript jit gives around 10% performance gain.
# Therefore we choose to play multiple games concurrently, each use only one thread.
def collect_self_play_data():
    # multi process
    pass



def train_step():
    pass

def evaluate():
    pass

if __name__ == '__main__':


    torch.set_num_threads(1)

    model = torch.jit.script(Model(121)) # gives around 10% performance gain

    self_play("standard", model)
