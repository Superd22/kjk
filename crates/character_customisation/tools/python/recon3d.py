"""Dense 3D face reconstruction with 3DDFA_V2, without its expression, so a smile or a parted mouth does not reach the fit."""
import os, sys
import numpy as np
import yaml

np.long = np.int64  # 3DDFA_V2 still uses this alias, which NumPy 2 removed
REPO = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "likeness-fit-data", "models", "3DDFA_V2")
sys.path.insert(0, REPO)
from TDDFA import TDDFA  # noqa: E402
from utils.tddfa_util import _parse_param  # noqa: E402


class Recon:
    def __init__(self):
        cfg = yaml.safe_load(open(os.path.join(REPO, "configs", "mb1_120x120.yml")))
        cfg["checkpoint_fp"] = os.path.join(REPO, cfg["checkpoint_fp"])
        cfg["bfm_fp"] = os.path.join(REPO, cfg["bfm_fp"])
        self.t = TDDFA(gpu_mode=False, **cfg)
        self.bfm = self.t.bfm

    def run(self, rgb, points):
        """Takes the MediaPipe points for the face box. Returns the identity shape in the model's own frame, and the full fit in image pixels."""
        x0, y0 = points.min(0)
        x1, y1 = points.max(0)
        params, rois = self.t(np.ascontiguousarray(rgb[..., ::-1]), [[x0, y0, x1, y1, 1.0]], crop_policy="box")
        R, off, a_shp, a_exp = _parse_param(params[0])
        b = self.bfm
        ident = (b.u + b.w_shp @ a_shp).reshape(3, -1, order="F").T
        image = self.t.recon_vers(params, rois, dense_flag=True)[0].T
        return {"identity": ident, "image": image, "R": R, "alpha": a_shp.ravel(), "expression": a_exp.ravel()}
