import torch

class Model(torch.nn.Module):
    def __init__(self, board_size):
        super(Model, self).__init__()
        self.embedding = torch.nn.Embedding(board_size*2, 192)
        self.encoder = torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(192, nhead=6, dim_feedforward=256), 8)
        self.pick_decoder = torch.nn.Sequential(
            torch.nn.Linear(192, 256),
            torch.nn.ReLU(),
            torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768), 2),
            torch.nn.Linear(256, 1),
        )
        self.move_decoder = torch.nn.Sequential(
            torch.nn.Linear(192, 256),
            torch.nn.ReLU(),
            torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768), 2),
            torch.nn.Linear(256, board_size),
        )
        self.value_decoder = torch.nn.Sequential(
            torch.nn.Linear(192, 256),
            torch.nn.ReLU(),
            torch.nn.TransformerEncoder(torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768), 2),
        )
        self.value_decoder_final = torch.nn.Linear(256, 1)

    def forward(self, pieces, pick_mask, move_mask):
        embeddings = self.embedding(pieces)
        embeddings = self.encoder(embeddings.permute(1, 0, 2)) # (seq, batch, dim)
        pick_logits = self.pick_decoder(embeddings) # (seq, batch, 1)
        pick_mask = (pick_mask - 1) * 100 # -100 is good enough to simulate -Inf in logits
        pick_logsoftmax = torch.nn.functional.log_softmax(torch.transpose(torch.squeeze(pick_logits, 2), 0, 1) * pick_mask, 1) # (batch, seq)
        move_logits = self.move_decoder(embeddings) # (seq, batch, board)
        move_mask = (move_mask - 1) * 100 # -100 is good enough to simulate -Inf in logits
        move_logsoftmax = torch.nn.functional.log_softmax(torch.transpose(move_logits, 0, 1) * move_mask, 2) # (batch, seq, board)
        value_before_pooling = self.value_decoder(embeddings).permute(1, 2, 0)
        value = torch.squeeze(self.value_decoder_final(torch.squeeze(torch.nn.functional.avg_pool1d(value_before_pooling, pieces.size()[1]), 2)), 1)
        return pick_logsoftmax, move_logsoftmax, value
