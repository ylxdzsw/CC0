import torch
import numpy as np

class Model(torch.nn.Module):
    def __init__(self, board_size):
        super(Model, self).__init__()
        self.embedding = torch.nn.Embedding(2 + board_size * 2, 256)
        self.encoder = torch.nn.Sequential(
            torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=384),
            torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=384),
            torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=384),
            torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=384),
        )
        self.decoder = torch.nn.Linear(256, 1)

    def forward(self, x):
        x = self.embedding(x)
        x = self.encoder(x.permute(1, 0, 2)) # (1 + 2*n_pieces, batch, hidden)
        x = x[0, :, :] # (batch, hidden)
        x = self.decoder(x) # (batch, 1)
        return torch.squeeze(x, 1)

def encode_game(game):
    x = [0] if game.is_p1_moving_next() else [1]

    for piece in game.p1_pieces():
        x.append(piece + 2)

    for piece in game.p2_pieces():
        x.append(piece + 2 + game.board_size)

    return x

def encode_child(game, child_pieces):
    x = [1] if game.is_p1_moving_next() else [0]

    for piece in child_pieces[:game.n_pieces]:
        x.append(piece + 2)

    for piece in child_pieces[game.n_pieces:]:
        x.append(piece + 2 + game.board_size)

    return x
