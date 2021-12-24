import torch
import numpy as np
from multiprocessing import Pool
from api import Game, MCTS, set_random_seed
from utils import save, load
from model import Model, encode_input

# playout a game and collect data for training [(pieces, masks, probs, result)]
# result means the winning rate (in -1 ~ 1 scale) of the player that is going to take action acroding to action_probs
# TODO: augmentation by horizontal fliping?
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

    data = [] # (pieces, masks, probs)

    while True:
        status = game.get_status()
        if status != 0:
            if status == 1:
                return [ (*x, 1 if i%2 == 0 else -1) for i, x in enumerate(data) ]
            if status == 2:
                return [ (*x, -1 if i%2 == 0 else 1) for i, x in enumerate(data) ]
            if status == 3:
                return [ (*x, 0) for x in data ]
            raise Exception("unknown status")

        mcts.playout(game, 1200 - mcts.total_visits())

        action_probs = mcts.get_action_probs(0.1)
        pieces, mask, probs = encode_input(game, action_probs)

        data.append((pieces, mask, probs))

        # state = game.dump()
        # value = mcts.root_value()
        # print(state, action_probs, value)

        old_pos, new_pos = mcts.sample_action(0.1, 0.1)
        game.do_move(old_pos, new_pos)
        mcts.chroot(old_pos, new_pos)

def worker_init(model_path):
    global model
    torch.set_num_threads(1)
    import os
    set_random_seed(os.getpid() * 7 + 39393)
    model = torch.jit.load(model_path).eval()

def worker_run(board_type):
    global model
    return self_play(board_type, model)

# inference performance: CUDA: 60 playouts/s, SingleThreadCPU: 80 playouts/s, MultiThreadCPU: 105 playouts/s but uses 40% of all 16 cores.
# TorchScript jit further gives around 10% performance gain.
# Therefore we choose to play multiple games concurrently, each use only one thread.
def collect_self_play_data(model, board_type="standard", n=1000):
    model.cpu().save('scripted_model.pt')
    with Pool(60, initializer=worker_init, initargs=('scripted_model.pt',)) as pool:
        data_batches = pool.map(worker_run, (board_type for _ in range(n)), chunksize=1)
    model.cuda()
    return [ x for batch in data_batches for x in batch ]

def random_batch(data, batch_size):
    return *(np.stack(d, axis=0) for d in zip(*( data[i] for i in np.random.randint(len(data), size=batch_size) ))),

def train(model, optimizer, data):
    model.train()

    acc = 0, 0
    for epoch in range(len(data) // 8):
        pieces, masks, probs, scores = ( torch.from_numpy(x).cuda() for x in random_batch(data, 64) )
        policy, value = model(pieces, masks)
        policy_loss = -torch.mean(torch.sum(probs * policy, 1))
        value_loss = torch.nn.functional.mse_loss(value, scores.float())

        (policy_loss + value_loss).backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), .6)
        optimizer.step()

        acc = acc[0] + policy_loss.item() / 1000, acc[1] + value_loss.item() / 1000
        if epoch % 1000 == 999:
            print(*acc)
            acc = 0, 0

# The argument can be either a checkpoint, or the board type
if __name__ == '__main__':
    import sys

    try:
        checkpoint = load(sys.argv[1])
        board_type = checkpoint["board_type"]
    except:
        board_type = sys.argv[1]

    dummy_game = Game(board_type)
    model = torch.jit.script(Model(dummy_game.board_size, dummy_game.n_pieces))
    optimizer = torch.optim.Adam(model.parameters(), lr=2e-5, weight_decay=2e-6)
    r = -1

    try:
        model.load_state_dict(checkpoint['model_state_dict'])
        optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        r = checkpoint['r']
    except:
        pass

    while True:
        r += 1

        try:
            data = load("data_{:03}".format(r))
        except:
            print("collecting data")
            data = collect_self_play_data(model, board_type, 600)
            save(data, "data_{:03}".format(r))

        print("load last 5 rounds data")
        for i in range(r-5, r):
            try:
                data.extend(load("data_{:03}".format(i)))
            except:
                print("skip data_{:03}".format(i))
                pass

        print("training model")
        train(model, optimizer, data)
        save({ 'r': r, 'board_type': board_type, 'model_state_dict': model.state_dict(), 'optimizer_state_dict': optimizer.state_dict() }, "model_{:03}".format(r))
