import torch
import numpy as np
from utils import load

data = load("data_000")
X = []
y = []

for encoded_state, value in data:
    X.append(encoded_state)
    y.append(value)

X_train = X[:-1000]
y_train = y[:-1000]
X_test = X[-1000:]
y_test = y[-1000:]

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

model = Transformer(73)

loss_fn = torch.nn.MSELoss(reduction="mean")
optimizer = torch.optim.Adam(model.parameters(), lr=2e-5)

n_epochs = 2
batch_size = 32

for epoch in range(n_epochs):
    for i in range(0, len(X_train), batch_size):
        x = torch.tensor(X_train[i:i + batch_size], dtype=torch.int32)
        y_ = torch.tensor(y_train[i:i + batch_size], dtype=torch.float32)

        p = torch.squeeze(model(x))
        loss = loss_fn(p, y_)
        print(loss.item())

        optimizer.zero_grad()
        loss.backward()
        optimizer.step()

p = torch.squeeze(model(torch.tensor(X_test, dtype=torch.float32)))
for p_, y_ in zip(p, y_test):
    print(p_.item(), y_)
