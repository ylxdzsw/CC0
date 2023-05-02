import torch
import numpy as np
from multiprocessing import Pool
from api import Game, set_random_seed, greedy, alphabeta
from utils import save, load
from model import Model

# otherwise it reports a problem that I don't bother to solve
torch.multiprocessing.set_sharing_strategy('file_system')

def gen_data(board_type, p1_type, p2_type):
    global target_model
    game = Game(board_type)

    data = [] # (encoded_inputs, final_result, weight)

    def _gen_data():
        key = game.key() # game will change afterwards

        if game.turn() >= 10 * game.n_pieces: # force end overly long games
            return -1, -1

        match game.get_status():
            case 0:
                player_type = p1_type if game.is_p1_moving_next() else p2_type

                match player_type:
                    case "greedy":
                        action = greedy(game, 0.1)
                        game.move_to(*action)
                    case "alphabeta":
                        action = alphabeta(game, 3)
                        game.move_to(*action)
                    case "model":
                        child_keys = game.expand()
                        batched_input = [ Model.encode_input(game, key) for key in child_keys ]
                        batched_input = torch.tensor(batched_input, dtype=torch.float)
                        probs = torch.sigmoid(model(batched_input))
                        if game.is_p2_moving_next():
                            probs = 1 - probs
                        probs = torch.softmax(probs / 0.1, 0)
                        i = torch.multinomial(probs, 1).item()
                        game.load_key(child_keys[i])

                p1win, ending_turn = _gen_data()

            case 1: # p1 win
                p1win, ending_turn = 1, key[0]

            case 2: # p2 win
                p1win, ending_turn = 0, key[0]

        if p1win != -1: # properly ended
            encoded_input = Model.encode_input(game, key)
            weight = 0.8 ** (ending_turn - key[0])
            data.append((encoded_input, p1win, weight))

        return p1win, ending_turn

    _gen_data()

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

def collect_data(target_model, spec):
    if target_model != None:
        target_model.cpu().save('scripted_model.pt')
        with Pool(128, initializer=worker_init, initargs=('scripted_model.pt',)) as pool:
            data_batches = pool.starmap(gen_data, spec, chunksize=4)
        target_model.cuda()
    else:
        with Pool(128, initializer=worker_init, initargs=(None,)) as pool:
            data_batches = pool.starmap(gen_data, spec, chunksize=4)

    return [ x for batch in data_batches for x in batch ]

def train(model, optimizer, data):
    model.train()

    class Dataset(torch.utils.data.Dataset):
        def __init__(self, data):
            self.data = data

        def __len__(self):
            return len(self.data)

        def __getitem__(self, index):
            x = np.array(self.data[index][0], dtype=np.float32)
            y = np.array(self.data[index][1], dtype=np.float32)
            w = np.array(self.data[index][2], dtype=np.float32)
            return x, y, w

    dataloader = torch.utils.data.DataLoader(Dataset(data), batch_size=1024, shuffle=True)
    i, total_loss = 0, 0
    for encoded_states, values, weights in dataloader:
        predicted_values = model(encoded_states.cuda())
        loss = torch.nn.functional.binary_cross_entropy_with_logits(predicted_values, values.cuda(), weight = weights.cuda())
        total_loss += loss.item() / 100
        optimizer.zero_grad() # important! default is accumulation
        loss.backward()
        # torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()

        i += 1
        if i % 100 == 0:
            print(total_loss, flush=True)
            total_loss = 0

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
    # optimizer = torch.optim.AdamW(model.parameters(), lr=5e-6, weight_decay=1e-2)
    optimizer = torch.optim.SGD(model.parameters(), lr=1e-5, weight_decay=1e-2)
    r = -1

    try:
        model.load_state_dict(checkpoint['model_state_dict'])
        # optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        r = checkpoint['r']
    except:
        pass

    while True:
        r += 1

        try:
            data = load("data_{:03}".format(r))
        except:
            print("collecting data")
            data = collect_data(None, (
                [(board_type, "greedy", "greedy")] * 10000 +
                [(board_type, "greedy", "alphabeta")] * 10000 +
                [(board_type, "alphabeta", "greedy")] * 10000 +
                [(board_type, "alphabeta", "alphabeta")] * 10000
            ))
            if r > 5:
                data += collect_data(model, (
                    [(board_type, "model", "greedy")] * 10000 +
                    [(board_type, "greedy", "model")] * 10000 +
                    [(board_type, "model", "alphabeta")] * 10000 +
                    [(board_type, "alphabeta", "model")] * 10000 +
                    [(board_type, "model", "model")] * 20000
                ))
            print(f"{len(data)} training data collected")
            save(data, "data_{:03}".format(r))

        if r > 5:
            print(f"load a random round history data")
            try:
                i = np.random.randint(0, r - 1)
                data.extend(load("data_{:03}".format(i)))
            except:
                print("reading data_{:03} failed".format(i))

        print("training model")
        train(model, optimizer, data)
        save({ 'r': r, 'board_type': board_type, 'model_state_dict': model.state_dict(), 'optimizer_state_dict': optimizer.state_dict() }, "model_{:03}".format(r))

        if r > 20:
            break
