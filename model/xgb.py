import numpy as np
from xgboost import XGBRegressor
from utils import load

data = load("data_000")
X = []
y = []

for encoded_state, value in data:
    x = np.zeros(1 + 2 * 73, dtype=np.int32)
    x[0] = encoded_state[0]
    for p in encoded_state[1:]:
        x[p] = 1
    X.append(x)
    y.append(value)

xgb = XGBRegressor(
    objective="reg:squarederror",
    n_estimators=4096,
    max_depth=16,
)

xgb.fit(X[:-1000], y[:-1000])

pred = xgb.predict(X[-1000:])
for p, v in zip(pred, y[-1000:]):
    print(p, v)
