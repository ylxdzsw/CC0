import torch
import numpy as np
from api import Game

def random_move(game):
    possible_moves = game.get_possible_moves()
    moveable = [ (pos, moves) for pos, moves in possible_moves if len(moves) != 0 ]
    assert len(moveable) > 0

    pos, moves = moveable[np.random.randint(len(moveable))]
    move = moves[np.random.randint(len(moves))]

    game.do_move(pos, move)

game = Game()
for i in range(200):
    random_move(game)
    status = game.get_status()
    if status != 0:
        print(status, i)
        break

