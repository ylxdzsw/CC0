# proof of concept: predict the possible moves

import torch
import numpy as np
import sys
from utils import save, load, normalize
from environment import Environment

try:
    data = load("data")
except:
    data = []

    while len(data) < 2000000:
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

# no positional encoding or mask used, so the input is unordered
class TransformerClassifier(torch.nn.Module):
    def __init__(self, board_size):
        super(TransformerClassifier, self).__init__()
        self.embedding = torch.nn.Embedding(board_size*3, 256)
        encoder_layer = torch.nn.TransformerEncoderLayer(256, nhead=8, dim_feedforward=768)
        self.encoder = torch.nn.TransformerEncoder(encoder_layer, 12)
        self.decoder = torch.nn.Linear(256, board_size)
        self.activation = torch.nn.Sigmoid()

    def forward(self, X):
        x = self.embedding(X)
        x = self.encoder(x.permute(1, 0, 2))
        x = self.decoder(x[0, :, :])
        x = self.activation(x)
        return x

training_set = data[:-5000]
test_set = data[-5000:]

# token size: 121 * 3. for the self pieces, oppo pieces, and the focused piece. The focuses piece always feed first.
def get_batch(dataset, batch_size, start=-1):
    X = np.zeros((batch_size, 21), dtype=np.int32)
    y = np.zeros((batch_size, 121), dtype=np.float32)

    for i in range(batch_size):
        if start < 0:
            index = np.random.randint(len(dataset))
        else:
            index = start + i
        self_pieces, oppo_pieces, pos, moves = dataset[index]

        X[i, 0] = pos
        for j, p in enumerate([p for p in self_pieces if p != pos]):
            X[i, j+1] = p + 121
        for j, p in enumerate(oppo_pieces):
            X[i, j+11] = p + 121*2
        for p in moves:
            y[i, p] = 1

    return X, y

model = TransformerClassifier(121).cuda()
loss = torch.nn.BCELoss()
optimizer = torch.optim.Adam(model.parameters(), lr=1e-5)
best = .1
epoch = 0

# a note about training: train shallow (8 layers, 3e-5 learning rate) model first. When it converges, deepen the model and load the parital parameters and continue training with small learing rate
try:
    checkpoint = load(sys.argv[1])
    model.load_state_dict(checkpoint['model_state_dict'])
    optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
    epoch = checkpoint['epoch']
    best = checkpoint['acc']
except:
    pass

while epoch < 800000:
    model.train()
    X, y = get_batch(training_set, 128)
    p = model(torch.from_numpy(X).cuda())
    L = loss(p, torch.from_numpy(y).cuda())
    L.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), .6)
    optimizer.step()
    epoch += 1
    if epoch % 200 == 0:
        model.eval()
        with torch.no_grad():
            X, y = get_batch(test_set, 5000, 0)
            p = model(torch.from_numpy(X).cuda())
            p = (p > .5).cpu().numpy()
            acc = sum(np.sum(p == y, 1) == 121) / 5000
            print("epoch {}, loss {:#.3g}, acc {:#.3g}".format(epoch, L.item(), acc))
            if acc > best:
                save({
                    'epoch': epoch,
                    'model_state_dict': model.state_dict(),
                    'optimizer_state_dict': optimizer.state_dict(),
                    'loss': L.item(),
                    'acc': acc
                }, "checkpoint_{:#.3g}".format(acc))
                best = acc
                # torch.jit.script(model).save("scripted_model")
