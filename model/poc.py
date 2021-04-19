# proof of concept: predict the possible moves

import torch
import numpy as np
from utils import save, load, normalize
from environment import Environment

try:
    data = load("data")
except:
    data = []

    while len(data) < 200000:
        env = Environment()
        for i in range(200):
            env.random_move()
            if i > 50: # only starts from there
                possible_moves = env.get_possible_moves()
                p = normalize([len(moves) for pos, moves in possible_moves]) # prefer pieces that have more possible moves
                i = np.random.choice(range(len(possible_moves)), p=p)
                self_pieces, oppo_pieces = env.dump()
                data.append((self_pieces, oppo_pieces, possible_moves[i][0], possible_moves[i][1]))

    save(data, "data")

# tokens are feed in [<CLS>, self_piece_pos_1, self_piece_pos_2, ..., oppo_piece_pos_1, oppo_piece_pos_2, ...]
# no positiontal encoding or mask used, so the input is unordered
class TransformerClassifier(torch.nn.Module):
    def __init__(self, board_size):
        super(TransformerClassifier, self).__init__()
        self._board_size = board_size
        self.embedding = torch.nn.Embedding(board_size+1, 192-3)
        encoder_layer = torch.nn.TransformerEncoderLayer(192, nhead=6, dim_feedforward=256)
        self.encoder = torch.nn.TransformerEncoder(encoder_layer, 8)
        self.decoder = torch.nn.Linear(192, board_size)
        self.activation = torch.nn.Sigmoid()

    def forward(self, pieces, masks):
        x = self.embedding(pieces)
        x = torch.cat((x, masks), 2)
        x = self.encoder(x.permute(1, 0, 2))
        x = self.decoder(x[0, :, :])
        x = self.activation(x)
        return x

training_set = data[:-5000]
test_set = data[-5000:]

def get_batch(dataset, batch_size, start=-1):
    pieces = np.zeros((batch_size, 21), dtype=np.int32)
    masks = np.zeros((batch_size, 21, 3), dtype=np.int32)
    y = np.zeros((batch_size, 121), dtype=np.float32)

    for i in range(batch_size):
        if start < 0:
            index = np.random.randint(len(dataset))
        else:
            index = start + i
        self_pieces, oppo_pieces, pos, moves = dataset[index]
        pieces[i, 0] = 121 # the special token
        for j, p in enumerate(self_pieces):
            pieces[i, j+1] = p
            masks[i, j+1, 0] = 1
            if p == pos:
                masks[i, j+1, 2] = 1
        for j, p in enumerate(oppo_pieces):
            pieces[i, j+11] = p
            masks[i, j+11, 1] = 1
            if p == pos:
                masks[i, j+11, 2] = 1
        for p in moves:
            y[i, p] = 1

    return pieces, masks, y

model = TransformerClassifier(121).cuda()
loss = torch.nn.BCELoss()
optimizer = torch.optim.Adam(model.parameters(), lr=3e-5)
best = 0

for epoch in range(50000):
    model.train()
    pieces, masks, y = get_batch(training_set, 256)
    p = model(torch.from_numpy(pieces).cuda(), torch.from_numpy(masks).cuda())
    L = loss(p, torch.from_numpy(y).cuda())
    L.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), .6)
    optimizer.step()
    if epoch % 100 == 0:
        model.eval()
        with torch.no_grad():
            pieces, masks, y = get_batch(test_set, 5000, 0)
            p = model(torch.from_numpy(pieces).cuda(), torch.from_numpy(masks).cuda())
            p = (p > .5).cpu().numpy()
            acc = sum(np.sum(p == y, 1) == 121) / 5000
            print("epoch {}, loss {:.3g}, acc {:.3g}".format(epoch, L.item(), acc))
            if epoch > 5000 and acc > best:
                torch.save({
                    'epoch': epoch,
                    'model_state_dict': model.state_dict(),
                    'optimizer_state_dict': optimizer.state_dict(),
                    'loss': L.item(),
                    'acc': acc
                }, "checkpoint_{:.3g}.pt".format(acc))
                best = acc

