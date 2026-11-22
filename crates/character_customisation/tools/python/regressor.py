"""The slider regressor: a ridge regression from MediaPipe points, 3DDFA_V2 identity numbers, gender and age to the symmetric face sliders.

It is trained on rendered random faces by train_ridge(). Real photos fall outside that training set, so the
lineup test, not the round trip, decides whether it helps.
"""
import os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from fit import DATA

PATH = os.path.join(DATA, "models", "ridge.npz")


def train_ridge(lam):
    from head import Head
    from idfit import Space
    head = Head(os.path.join(DATA, "likeness"))
    sp = Space(head)
    tr = np.load(os.path.join(DATA, "train", "train.npz"))
    X = np.concatenate([tr["X"], tr["M"][:, :2]], 1).astype(np.float64)
    Y = np.stack([sp.encode(v)[:sp.n_shape] for v in tr["V"]])
    mean, std, ym = X.mean(0), X.std(0) + 1e-6, Y.mean(0)
    A = (X - mean) / std
    W = np.linalg.solve(A.T @ A + lam * np.eye(A.shape[1]), A.T @ (Y - ym))
    np.savez(PATH, W=W, mean=mean, std=std, ym=ym, lam=lam)


class Regressor:
    def __init__(self, path=PATH):
        r = np.load(path)
        self.W, self.mean, self.std, self.ym = r["W"], r["mean"], r["std"], r["ym"]
        from recon3d import Recon
        self.recon = Recon()

    def predict_shape(self, rgb, det_result, gender, age):
        """Returns the symmetric face slider values, in the order of idfit.Space."""
        from gen_data import features
        from detect import IRIS_L, IRIS_R
        from fit import normalise
        p = det_result.points * [1, -1]
        lm = normalise(p[:468], p[IRIS_R], p[IRIS_L]).ravel()
        alpha = self.recon.run(rgb, det_result.points)["alpha"] / 1e5
        x = np.concatenate([lm, alpha, [gender, age]])
        return ((x - self.mean) / self.std) @ self.W + self.ym
