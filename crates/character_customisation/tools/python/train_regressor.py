"""Trains a regressor from what the detectors see to the slider values, on rendered random faces.

Usage: python -I train_regressor.py
"""
import os, sys
import numpy as np
import torch

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from fit import DATA
from head import Head
from idfit import Space
from test_roundtrip import face_error

TRAIN = os.path.join(DATA, "train")
MODEL = os.path.join(DATA, "models", "regressor.pt")


def inputs(X, M):
    return np.concatenate([X, M[:, :2]], 1).astype(np.float32)


class Net(torch.nn.Module):
    def __init__(self, n_in, n_out, mean, std):
        super().__init__()
        self.register_buffer("mean", torch.tensor(mean))
        self.register_buffer("std", torch.tensor(std))
        self.f = torch.nn.Sequential(torch.nn.Linear(n_in, 512), torch.nn.GELU(), torch.nn.Dropout(0.1), torch.nn.Linear(512, 512), torch.nn.GELU(), torch.nn.Dropout(0.1), torch.nn.Linear(512, n_out))

    def forward(self, x):
        return self.f((x - self.mean) / self.std)


def main():
    head = Head(os.path.join(DATA, "likeness"))
    sp = Space(head)
    tr, te = np.load(os.path.join(TRAIN, "train.npz")), np.load(os.path.join(TRAIN, "test.npz"))
    Xtr, Xte = inputs(tr["X"], tr["M"]), inputs(te["X"], te["M"])
    Ytr = np.stack([sp.encode(v)[:sp.n_shape] for v in tr["V"]]).astype(np.float32)
    mean, std = Xtr.mean(0), Xtr.std(0) + 1e-6

    # ridge regression as the baseline
    A = (Xtr - mean) / std
    lam = 30.0
    W = np.linalg.solve(A.T @ A + lam * np.eye(A.shape[1]), A.T @ (Ytr - Ytr.mean(0)))
    ridge_pred = ((Xte - mean) / std) @ W + Ytr.mean(0)

    torch.manual_seed(0)
    net = Net(Xtr.shape[1], Ytr.shape[1], mean, std)
    opt = torch.optim.AdamW(net.parameters(), lr=1e-3, weight_decay=1e-4)
    Xt, Yt = torch.tensor(Xtr), torch.tensor(Ytr)
    for epoch in range(300):
        net.train()
        perm = torch.randperm(len(Xt))
        for i in range(0, len(Xt), 256):
            b = perm[i:i + 256]
            loss = torch.nn.functional.mse_loss(net(Xt[b]), Yt[b])
            opt.zero_grad()
            loss.backward()
            opt.step()
    net.eval()
    with torch.no_grad():
        mlp_pred = net(torch.tensor(Xte)).numpy()
    torch.save(net.state_dict(), MODEL)
    np.savez(MODEL.replace(".pt", "_shape.npz"), n_in=Xtr.shape[1], n_out=Ytr.shape[1])

    avg = head.deform(head.target_weights(np.zeros(len(head.modifiers))))
    for label, pred in (("ridge regression", ridge_pred), ("neural network", mlp_pred)):
        rows = []
        for k in range(len(Xte)):
            g, a, *mix = te["M"][k]
            truth = head.deform(head.target_weights(te["V"][k], g, a, mix))
            z = np.concatenate([np.clip(pred[k], sp.lo[:sp.n_shape], sp.hi[:sp.n_shape]), sp.encode(np.zeros(len(head.modifiers)))[sp.n_shape:]])
            fitted = head.deform(head.target_weights(sp.decode(z)[0], g, a, mix))
            rows.append((*face_error(head, avg, truth), *face_error(head, fitted, truth)))
        r = np.array(rows)
        print(f"{label}: removes {100 * (1 - r[:, 2].mean() / r[:, 0].mean()):.0f}% of the difference in the photo plane and {100 * (1 - r[:, 3].mean() / r[:, 1].mean()):.0f}% in depth, over {len(r)} test faces.")


if __name__ == "__main__":
    main()
