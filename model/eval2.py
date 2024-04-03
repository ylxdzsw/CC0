import torch
import numpy as np
from tqdm import tqdm
from api import Game, greedy, alphabeta
from model import Model

import sys

def load_model(path_or_type):
    try:
        checkpoint = torch.load(path_or_type)
    except:
        model = path_or_type
    else:
        board_type = checkpoint['board_type']
        dummy_game = Game(board_type)
        model = torch.jit.script(Model(dummy_game.board_size).cuda())
        model.load_state_dict(checkpoint['model'])
        model.eval()

    return model

def run_action(game, model):
    match model:
        case "greedy":
            action = greedy(game, 0.1)
            game.move_to(*action)
        case "alphabeta":
            action = alphabeta(game, 3)
            game.move_to(*action)
        case _:
            original_key = game.key()
            child_keys = game.expand()
            batched_input = [ Model.encode_input(game, key) for key in child_keys ]
            batched_input = torch.tensor(batched_input, dtype=torch.float).cuda()
            predictions = model(batched_input).cpu()

            child_scores = []
            for i, key in enumerate(child_keys):
                game.load_key(key)
                if game.get_status() == 0:
                    child_scores.append(predictions[i].item())
                else:
                    child_scores.append(game.distance_diff_score())

            game.load_key(original_key)
            sign = 50 if game.is_p1_moving_next() else -50 # temperature: 0.02
            probs = torch.softmax(torch.tensor(child_scores) * sign, 0)

            entropy = -torch.sum(probs * torch.log(probs)).item()
            if not np.isnan(entropy):
                if game.is_p1_moving_next():
                    p1_entropy.append(entropy)
                else:
                    p2_entropy.append(entropy)

            game.load_key(child_keys[torch.multinomial(probs, 1).item()])

p1 = load_model(sys.argv[1])
p2 = load_model(sys.argv[2])

p1_win = 0
p2_win = 0

p1_entropy = []
p2_entropy = []

for _ in tqdm(range(200)):
    game = Game("small")
    while game.get_status() == 0:
        run_action(game, p1 if game.is_p1_moving_next() else p2)
        if game.turn() >= 10 * game.n_pieces:
            break

    match game.get_status():
        case 1:
            p1_win += 1
        case 2:
            p2_win += 1

print("p1 win rate:", p1_win / (p1_win + p2_win))
if len(p1_entropy) > 0:
    print("p1 entropy:", np.mean(p1_entropy))
if len(p2_entropy) > 0:
    print("p2 entropy:", np.mean(p2_entropy))
