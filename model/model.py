import torch
import numpy as np

class Transformer(torch.nn.Module):
    def __init__(self, board_size):
        super(Transformer, self).__init__()
        self.embedding = torch.nn.Embedding(2 + board_size * 2, 32)
        self.encoder = torch.nn.Sequential(
            torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
            # torch.nn.TransformerEncoderLayer(32, nhead=4, dim_feedforward=48),
        )
        self.decoder = torch.nn.Linear(32, 1)

    def forward(self, x):
        x = self.embedding(x)
        x = x.permute(1, 0, 2) # (1 + 2*n_pieces, batch, hidden)
        x = self.encoder(x) # (1 + 2*n_pieces, batch, hidden)
        x = x[0, :, :] # (batch, hidden)
        x = self.decoder(x) # (batch, 1)
        return torch.squeeze(x, 1)

    @classmethod
    def encode_game(_cls, game):
        x = [0] if game.is_p1_moving_next() else [1]

        for piece in game.p1_pieces():
            x.append(piece + 2)

        for piece in game.p2_pieces():
            x.append(piece + 2 + game.board_size)

        return x

    @classmethod
    def encode_child(_cls, game, child_pieces):
        x = [1] if game.is_p1_moving_next() else [0]

        for piece in child_pieces[:game.n_pieces]:
            x.append(piece + 2)

        for piece in child_pieces[game.n_pieces:]:
            x.append(piece + 2 + game.board_size)

        return x

class MLP(torch.nn.Module):
    def __init__(self, board_size):
        super(MLP, self).__init__()
        self.layers = torch.nn.Sequential(
            torch.nn.Linear(1 + 2 * board_size, 1024),
            torch.nn.ReLU(),
            torch.nn.Linear(1024, 4096),
            torch.nn.ReLU(),
            torch.nn.Linear(4096, 1024),
            torch.nn.ReLU(),
            torch.nn.Linear(1024, 1),
        )

    def forward(self, x):
        x = self.layers(x)
        return torch.squeeze(x, 1)

    @classmethod
    def encode_game(_cls, game):
        x = [0] * (1 + 2 * game.board_size)

        if game.is_p1_moving_next():
            x[0] = 1

        for piece in game.p1_pieces():
            x[1 + piece] = 1

        for piece in game.p2_pieces():
            x[1 + game.board_size + piece] = 1

        return x

    @classmethod
    def encode_child(_cls, game, child_pieces):
        x = [0] * (1 + 2 * game.board_size)

        if game.is_p2_moving_next():
            x[0] = 1

        for piece in child_pieces[:game.n_pieces]:
            x[1 + piece] = 1

        for piece in child_pieces[game.n_pieces:]:
            x[1 + game.board_size + piece] = 1

        return x

Model = MLP
