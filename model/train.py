import torch
import numpy as np
from multiprocessing import Pool
from api import Game, set_random_seed
from utils import save, load
from model import Model, encode_game, encode_child

torch.multiprocessing.set_sharing_strategy('file_system')

def gen_data(board_type, target_model):
    game = Game(board_type)

    data = [] # (encoded_inputs, state_value)

    while game.get_status() == 0:
        if game.turn() >= 8 * game.n_pieces: # force end overly long games
            break

        child_pieces, child_values, actions, terminals = game.expand()

        if target_model != None:
            for i, pieces in enumerate(child_pieces):
                if terminals[i]:
                    continue # use the true reward
                encoded = torch.unsqueeze(torch.tensor(encode_child(game, pieces)), 0)
                child_values[i] = target_model(encoded).item()

        probs_unnormalized = torch.tensor(child_values, dtype=torch.float)
        if game.is_p2_moving_next():
            probs_unnormalized = 1 - probs_unnormalized
        probs = torch.softmax(probs_unnormalized / 0.1, 0)

        if game.turn() >= 2 * game.n_pieces: # skip first several moves
            updated_value = (torch.tensor(child_values, dtype=torch.float) * probs).sum().item()
            if 0.48 < updated_value < 0.52:
                pass # skip near-draw games
            else:
                data.append((encode_game(game), updated_value))

        i = torch.multinomial(probs, 1).item()
        from_pos, to_pos = actions[i]
        game.move_to(from_pos, to_pos)

    return data

def worker_init(model_path):
    import os
    set_random_seed(os.getpid() * 7 + 39393)

    global target_model
    if model_path is None:
        target_model = None
    else:
        torch.set_num_threads(1)
        target_model = torch.jit.load(model_path).eval()

def worker_run(board_type):
    global target_model
    return gen_data(board_type, target_model)

def collect_data(target_model, n, board_type="standard"):
    if target_model != None:
        target_model.cpu().save('scripted_model.pt')
        with Pool(128, initializer=worker_init, initargs=('scripted_model.pt',)) as pool:
            data_batches = pool.map(worker_run, (board_type for _ in range(n)), chunksize=1)
        target_model.cuda()
    else:
        with Pool(128, initializer=worker_init, initargs=(None,)) as pool:
            data_batches = pool.map(worker_run, (board_type for _ in range(n)), chunksize=1)

    return [ x for batch in data_batches for x in batch ]

def train(model, optimizer, data):
    model.train()

    class Dataset(torch.utils.data.Dataset):
        def __init__(self, data):
            self.data = data

        def __len__(self):
            return len(self.data)

        def __getitem__(self, index):
            x = np.array(self.data[index][0], dtype=np.int64)
            y = np.array(self.data[index][1], dtype=np.float32)
            return x, y

    dataloader = torch.utils.data.DataLoader(Dataset(data), batch_size=1024, shuffle=True)
    for encoded_states, values in dataloader:
        predicted_values = model(encoded_states.cuda())
        loss = torch.nn.functional.mse_loss(predicted_values, values.cuda())
        print(loss.item())
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), .6)
        optimizer.step()

# The argument can be either a checkpoint, or the board type
if __name__ == '__main__':
    import sys

    try:
        checkpoint = load(sys.argv[1])
        board_type = checkpoint["board_type"]
    except:
        board_type = sys.argv[1]

    dummy_game = Game(board_type)
    model = torch.jit.script(Model(dummy_game.board_size).cuda())
    optimizer = torch.optim.Adam(model.parameters(), lr=1e-4)
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
            if r == 0:
                data = collect_data(None, 1000000, board_type)
            else:
                data = collect_data(model, 50000, board_type)
            save(data, "data_{:03}".format(r))

        print(f"load last 5 rounds data")
        for i in range(r-5, r):
            try:
                data.extend(load("data_{:03}".format(i)))
            except:
                print("skip data_{:03}".format(i))
                pass

        print("training model")
        train(model, optimizer, data)
        save({ 'r': r, 'board_type': board_type, 'model_state_dict': model.state_dict(), 'optimizer_state_dict': optimizer.state_dict() }, "model_{:03}".format(r))

        if r > 100:
            break
