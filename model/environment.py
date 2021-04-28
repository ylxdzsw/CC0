import numpy as np
from api import Game

class Environment(Game):
    def __init__(self, *args):
        super().__init__(*args)
        self.records = []

    def do_move(self, old_pos, new_pos):
        self.records.append((old_pos, new_pos))
        super().do_move(old_pos, new_pos)

    def random_move(self):
        possible_moves = self.get_possible_moves()
        assert len(possible_moves) > 0

        pos, moves = possible_moves[np.random.randint(len(possible_moves))]
        move = moves[np.random.randint(len(moves))]

        self.do_move(pos, move)

    def replay(self, records):
        pass
