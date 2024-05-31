from matplotlib import patches
from eight_ball import _Ball

class Ball(_Ball):
    def __init__(self, **kwargs):
         self._patch = patches.Circle(self.pos, self.r)

    def get_patch(self):
        """
        Returns the current patch with any timestep limitations, after moving
        the centre.
        """
        self._patch.center = self.pos
        return self._patch

