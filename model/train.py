import torch
import numpy as np
from api import Game, MCTS
from utils import save, load
from model import Model, encode_input

# playout a game and return [(state, action_probs, value)] of each selected node
def self_play(board_type, model):

    def policy_fun(game):
        return model(*encode_input(game))
    game = Game(board_type)
    mcts = MCTS(policy_fun)



def collect_self_play_data():
    # multi process
    # model.eval()
    # with torch.no_grad():
    pass

def rotate_and_flip():
    pass

def train_step():
    pass

def evaluate():
    pass

model = Model()
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
                torch.jit.script(model).save("scripted_model")


