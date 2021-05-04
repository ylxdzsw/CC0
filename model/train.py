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
            return record, status

        mcts.playout(game, 800 - mcts.total_visits())

        action_probs = mcts.get_action_probs(1e-3)
        pieces, mask, probs = encode_input(game, action_probs)

        data.append((pieces, mask, probs))

        # state = game.dump()
        # value = mcts.root_value()
        # print(state, action_probs, value)

        old_pos, new_pos = mcts.sample_action(0.1, 0.1) # the temperature used for self-play is not the same as for collecting trace
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
def collect_self_play_data(model, n=1000):
    model.cpu().save('scripted_model.pt')
    with Pool(8, initializer=worker_init, initargs=('scripted_model.pt',)) as pool:
        data_batches = pool.map(worker_run, ('small' for _ in range(n)))
    return [ x for batch in data_batches for x in batch ]

def random_batch(data, batch_size):
    return *(np.stack(d, axis=0) for d in zip(*( data[i] for i in np.random.randint(len(data), size=batch_size) ))),

def train(model, optimizer, data):
    model.train()

    acc = 0, 0
    for epoch in range(2000):
        # pieces, masks, probs, scores = ( torch.from_numpy(x).cuda() for x in random_batch(data, 32) )
        pieces, masks, probs, scores = ( torch.from_numpy(x) for x in random_batch(data, 32) )
        policy, value = model(pieces, masks)
        policy_loss = -torch.mean(torch.sum(probs * policy, 1))
        value_loss = torch.nn.functional.mse_loss(value, scores.float())

        (policy_loss + value_loss).backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), .6)
        optimizer.step()
        epoch += 1

        acc = acc[0] + policy_loss.item() / 100, acc[1] + value_loss.item() / 100
        if epoch % 100 == 99:
            print(*acc)
            acc = 0, 0

def evaluate():
    pass

if __name__ == '__main__':
    import sys

    model = torch.jit.script(Model(73))
    optimizer = torch.optim.Adam(model.parameters(), lr=2e-5, weight_decay=1e-6)
    r = -1

    try:
        checkpoint = load(sys.argv[1])
        model.load_state_dict(checkpoint['model_state_dict'])
        optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        r = checkpoint['r']
    except:
        pass

    while True:
        r += 1

        try:
            data = load("data_{}".format(r))
        except:
            print("collecting data")
            data = collect_self_play_data(model, 100)
            save(data, "data_{}".format(r))

        print("load last 5 rounds data")
        for i in range(r-5, r):
            if i < 0:
                continue
            data.extend(load("data_{}".format(i)))

        print("training model")
        # model.cuda()
        train(model, optimizer, data)
        save({ 'r': r, 'model_state_dict': model.state_dict(), 'optimizer_state_dict': optimizer.state_dict() }, "model_{}".format(r))
