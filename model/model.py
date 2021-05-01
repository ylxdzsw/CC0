import torch
import numpy as np

class Model(torch.nn.Module):
    def __init__(self, board_size):
        super(Model, self).__init__()
        self.embedding = torch.nn.Embedding(board_size * 4, 192)
        self.shared_encoder = torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(192, nhead=6, dim_feedforward=256), 8)
        self.policy_encoder = torch.nn.Sequential(
            torch.nn.Linear(192, 256),
            torch.nn.ReLU(),
            torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768), 2),
        )
        self.policy_decoder = torch.nn.Linear(256, board_size)
        self.value_encoder = torch.nn.Sequential(
            torch.nn.Linear(192, 256),
            torch.nn.ReLU(),
            torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768), 2),
        )
        self.value_decoder = torch.nn.Linear(256, 1)

    def forward(self, pieces, mask):
        mask = (mask - 1) * 100 # -100 is good enough to be considered as -Inf in logits

        embeddings = self.embedding(pieces)
        embeddings = self.shared_encoder(embeddings.permute(1, 0, 2)) # (2*n_pieces, batch, hidden)

        policy = self.policy_encoder(embeddings) # (2*n_pieces, batch, hidden)
        policy = policy[:pieces.size()[1]//2, :, :].transpose(0, 1) # (n_pieces, batch, hidden)
        policy = self.policy_decoder(policy).reshape((pieces.size()[0], -1)) # (batch, n_pieces * board)
        policy = torch.nn.functional.log_softmax(policy + mask, 1) # (batch, n_pieces * board)

        value = self.value_encoder(embeddings) # (2*n_pieces, batch, hidden)
        value = value.permute(1, 2, 0) # (batch, hidden, 2*n_pieces)
        value = torch.squeeze(torch.nn.functional.max_pool1d(value, pieces.size()[1]), 2) # (batch, hidden)
        value = torch.squeeze(self.value_decoder(value), 1) # (batch, )

        return policy, value

def encode_input(game, action_probs=None):
    board_size, n_pieces = game.board_size, game.n_pieces
    player, first_pieces, second_pieces = game.dump()
    player_offset = 0 if player == 1 else 2 * board_size
    if player == 1:
        self_pieces, oppo_pieces = first_pieces, second_pieces
    elif player == 2:
        self_pieces, oppo_pieces = second_pieces, first_pieces
    possible_moves = game.all_possible_moves()

    pieces = np.zeros(2 * n_pieces, dtype=np.int32)
    mask = np.zeros(n_pieces * board_size, dtype=np.int32)

    pieces[:n_pieces] = np.array(self_pieces) + player_offset
    pieces[n_pieces:] = np.array(oppo_pieces) + board_size + player_offset

    for pos, moves in possible_moves:
        i = self_pieces.index(pos)
        for j in moves:
            mask[i * board_size + j] = 1

    if action_probs is None:
        return pieces, mask

    probs = np.zeros(n_pieces * board_size, dtype=np.float32)
    for old_pos, new_pos, prob in action_probs:
        i = self_pieces.index(old_pos)
        probs[i * board_size + new_pos] = 1
    return pieces, mask, probs
