from matplotlib import patches
from eight_ball import _Ball


class Ball(_Ball):
    def __init__(self, **kwargs):
        pass

    @property
    def patch(self) -> patches.Circle:
        return patches.Circle(self.pos, radius=self.r)


class Container(Ball):
    @property
    def patch(self) -> patches.Circle:
        return patches.Circle(self.pos,
                              radius=-self.r,
                              facecolor="white",
                              edgecolor="black",
                              zorder=0)
