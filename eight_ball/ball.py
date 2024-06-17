from typing import Tuple
from matplotlib import patches
from eight_ball import _Ball


class Ball(_Ball):
    def __init__(self, **kwargs):
        pass

    @property
    def patch(self) -> patches.Circle:
        return patches.Circle(self.pos, radius=self.r)

    def pair_hash(self, other) -> Tuple[Tuple[float, float], Tuple[float, float]]:
        return self.vel, other.vel


class Container(Ball):
    @property
    def patch(self) -> patches.Circle:
        return patches.Circle(self.pos,
                              radius=-self.r,
                              facecolor="white",
                              edgecolor="black",
                              zorder=0)
