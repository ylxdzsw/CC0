import torch
import numpy as np
from multiprocessing import Pool
from api import Game, set_random_seed, greedy, alphabeta
from model import Model
from tqdm import tqdm

# otherwise it reports a problem that I don't bother to solve
torch.multiprocessing.set_sharing_strategy('file_system')

def gen_data(board_type):
    global target_model
    game = Game(board_type)

    data = [] # (encoded_inputs, score)
    visited = set()

    while True:
        original_key = game.key()
        visited.add(tuple(original_key))

        child_keys = game.expand()
        if len(child_keys) == 0:
            break # finished

        if game.turn() >= 10 * game.n_pieces:
            return [] # too long

        child_keys = [ key for key in child_keys if tuple(key) not in visited ]
        if len(child_keys) == 0:
            return [] # stuck

        if target_model != None:
            batched_input = [ Model.encode_input(game, key) for key in child_keys ]
            batched_input = torch.tensor(batched_input, dtype=torch.float)
            predictions = target_model(batched_input)

        child_scores = []
        for i, key in enumerate(child_keys):
            game.load_key(key)
            if target_model != None and game.get_status() == 0:
                child_scores.append(predictions[i].item())
            else:
                child_scores.append(game.distance_diff_score())

        game.load_key(original_key)

        if game.turn() >= 4: # skip the first two moves
            selection_index = -1 if game.is_p1_moving_next() else 0
            data.append((Model.encode_input(game, original_key), sorted(child_scores)[selection_index]))

        sign = 50 if game.is_p1_moving_next() else -50 # temperature: 0.02
        probs = torch.softmax(torch.tensor(child_scores) * sign, 0)
        game.load_key(child_keys[torch.multinomial(probs, 1).item()])

    return data

def worker_init(target_model_path):
    import os
    set_random_seed(os.getpid() * 97 + 39393)
    torch.manual_seed(os.getpid() * 97 + 39393)

    global target_model
    if target_model_path is None:
        target_model = None
    else:
        torch.set_num_threads(1)
        target_model_checkpoint = torch.load(target_model_path, map_location='cpu')
        board_type = target_model_checkpoint["board_type"]
        dummy_game = Game(board_type)
        target_model = torch.jit.script(Model(dummy_game.board_size))
        target_model.load_state_dict(target_model_checkpoint['model'])
        target_model.eval()

def collect_data(target_model_path, spec):
    with Pool(8, initializer=worker_init, initargs=(target_model_path,)) as pool:
        data_batches = pool.starmap(gen_data, tqdm(spec), chunksize=4)

    return [ x for batch in data_batches for x in batch ]

def train(model, optimizer, data):
    class Dataset(torch.utils.data.Dataset):
        def __init__(self, data):
            self.data = data

        def __len__(self):
            return len(self.data)

        def __getitem__(self, index):
            x = np.array(self.data[index][0], dtype=np.float32)
            y = np.array(self.data[index][1], dtype=np.float32)
            return x, y

    dataloader = torch.utils.data.DataLoader(Dataset(data), batch_size=64, shuffle=True)
    i, total_loss = 0, 0
    for encoded_states, scores in dataloader:
        predicted_scores = model(encoded_states.cuda())
        loss = torch.nn.functional.mse_loss(predicted_scores, scores.cuda())
        total_loss += loss.item() / 1000
        optimizer.zero_grad() # important! default is accumulation
        loss.backward()
        # torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()

        i += 1
        if i % 1000 == 0:
            print(total_loss, flush=True)
            total_loss = 0

def main():
    import sys

    if len(sys.argv) != 3:
        print("Usage: train2.py <target_model_checkpoint|board_type> <training_model_checkpoint>")
        sys.exit(1)

    try:
        target_model_checkpoint = torch.load(sys.argv[1])
        board_type = target_model_checkpoint["board_type"]
        target_model_path = sys.argv[1]
    except:
        board_type = sys.argv[1]
        target_model_path = None

    dummy_game = Game(board_type)
    model = torch.jit.script(Model(dummy_game.board_size).cuda())
    optimizer = torch.optim.AdamW(model.parameters(), lr=5e-6, weight_decay=1e-2)
    # optimizer = torch.optim.SGD(model.parameters(), lr=5e-6, weight_decay=1e-2)

    try:
        checkpoint = torch.load(sys.argv[2])
        model.load_state_dict(checkpoint['model'])
        optimizer.load_state_dict(checkpoint['optimizer'])
        print("model loaded")
    except:
        try:
            model.load_state_dict(target_model_checkpoint['model'])
            optimizer.load_state_dict(target_model_checkpoint['optimizer'])
            print("model loaded from target model")
        except:
            print("model initialized from scratch")

    while True:
        print("collecting data")
        data = collect_data(target_model_path, [(board_type,)] * 16384)
        print("training model")
        model.train()
        train(model, optimizer, data)
        print("saving checkpoint")
        checkpoint = {
            'board_type': board_type,
            'model': model.state_dict(),
            'optimizer': optimizer.state_dict(),
        }
        torch.save(checkpoint, sys.argv[2])

# The first argument can be either a checkpoint to target model, or the board type
# The second argument is the checkpoint of the training model
if __name__ == '__main__':
    main()
