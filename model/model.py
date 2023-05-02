import torch
import numpy as np

class Transformer(torch.nn.Module):
    def __init__(self, board_size):
        super(Transformer, self).__init__()
        self.embedding = torch.nn.Embedding(2 + board_size * 2, 256)
        self.encoder = torch.nn.Sequential(
            torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
            # torch.nn.TransformerEncoderLayer(256, nhead=4, dim_feedforward=384),
        )
        self.decoder = torch.nn.Linear(256, 1)

    def forward(self, x):
        x = self.embedding(x)
        x = x.permute(1, 0, 2) # (1 + 2*n_pieces, batch, hidden)
        x = self.encoder(x) # (1 + 2*n_pieces, batch, hidden)
        x = x[0, :, :] # (batch, hidden)
        x = self.decoder(x) # (batch, 1)
        return torch.squeeze(x, 1)

    @classmethod
    def encode_input(_cls, game, key = None):
        if key == None:
            x = [0] if game.is_p1_moving_next() else [1]

            for piece in game.p1_pieces():
                x.append(piece + 2)

            for piece in game.p2_pieces():
                x.append(piece + 2 + game.board_size)
        else:
            x = [0] if key[0] % 2 == 0 else [1]

            for piece in key[1:game.board_size+1]:
                x.append(piece + 2)

            for piece in key[game.board_size+1:]:
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
    def encode_input(_cls, game, key = None):
        x = [0] * (1 + 2 * game.board_size)

        if key == None:
            if game.is_p1_moving_next():
                x[0] = 1

            for piece in game.p1_pieces():
                x[1 + piece] = 1

            for piece in game.p2_pieces():
                x[1 + game.board_size + piece] = 1
        else:
            if key[0] % 2 == 0:
                x[0] = 1

            for piece in key[1:game.board_size+1]:
                x[1 + piece] = 1

            for piece in key[game.board_size+1:]:
                x[1 + game.board_size + piece] = 1

        return x

class Block(torch.nn.Module):
    def __init__(self, feature, hidden, dropout = 0.2):
        super(Block, self).__init__()
        self.layers = torch.nn.Sequential(
            torch.nn.ReLU(),
            torch.nn.Linear(feature, hidden),
            torch.nn.ReLU(),
            torch.nn.Dropout(dropout),
            torch.nn.Linear(hidden, feature),
        )

    def forward(self, x):
        return self.layers(x) + x

class RMLP(torch.nn.Module):
    def __init__(self, board_size):
        super(RMLP, self).__init__()
        self.encoder = torch.nn.Linear(1 + 2 * board_size, 1024)
        self.blocks = torch.nn.Sequential(
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            Block(1024, 256),
            torch.nn.ReLU(),
        )
        self.decoder = torch.nn.Linear(1024, 1)

    def forward(self, x):
        x = self.encoder(x)
        x = self.blocks(x)
        x = self.decoder(x)
        return torch.squeeze(x, 1)

    @classmethod
    def encode_input(_cls, game, key = None):
        return MLP.encode_input(game, key)

Model = RMLP
