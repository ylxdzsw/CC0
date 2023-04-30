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

class MLP(torch.nn.Module):
    def __init__(self, board_size):
        super(MLP, self).__init__()
        self.layers = torch.nn.Sequential(
            torch.nn.Linear(1 + 2 * board_size, 1024),
            torch.nn.ReLU(),
            torch.nn.Linear(1024, 256),
            torch.nn.ReLU(),
            torch.nn.Linear(256, 1),
        )

    def forward(self, x):
        return torch.squeeze(self.layers(x), 1)

Model = MLP

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

# turn transformer encode into mlp encode
def re_encode(encoded, board_size):
    x = np.zeros(1 + 2 * board_size, dtype=np.float32)
    x[0] = encoded[0]
    for p in encoded[1:]:
        x[p] = 1
    return x
