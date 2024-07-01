from typing import List, Tuple
from numpy import tanh
import matplotlib.pyplot as plt
from matplotlib import patches
from eight_ball import _Simulation
from eight_ball.ball import Ball

class Simulation(_Simulation):
    def __init__(self, radius: float): 
        self.container_radius = radius

    def comic_strip(self, collisions: int):
        width = floor_sqrt(collisions)
        height = ceildiv(collisions, width)
        _, axs = plt.subplots(height, width)
        draw(axs[0][0], self.get_balls(), self.container_radius, 0.)
        for c in range(1, collisions):
            self.next_collision()
            i = c // width
            j = c % width
            draw(axs[i][j], self.get_balls(), self.container_radius, self.global_time)

        plt.tight_layout()
        plt.show()


def draw(ax: plt.Axes, balls: List[Ball], radius: float, time: float):
    """
    Draw the simulation for debugging purposes.
    """
    container_patch = patches.Circle((0., 0.),
                          radius=radius,
                          facecolor="white",
                          edgecolor="black",
                          zorder=0)
    ax.add_patch(container_patch)
    for ball in balls:
        ball_patch = patches.Circle(ball.pos, radius=ball.r)
        ax.add_patch(ball_patch)
        # x, y = ball.pos
        # dx, dy = ball.vel
        if ball.vel != (0., 0.):
            ax.arrow(*ball.pos, *hyp_length(ball.vel), head_width=0.05, head_length=0.03)
    ax.set_title(f"{time=:3f}")


def ceildiv(a, b):
    return -(a // -b)

def hyp_length(r: Tuple[float, float]) -> Tuple[float, float]:
    # length = (r[0]**2 + r[1]**2) ** 0.5
    # tanh_length = tanh(length)
    return 0.4 * tanh(r)
    
    

def floor_sqrt(x: int) -> int:
    """
    Calculate the greatest integer, `n`, such that `n^2` is not greater than x.
    """
    left = 1
    right = x
    while right - left > 1:
        midpoint = (left + right) // 2
        m_squared = midpoint ** 2
        if m_squared == x:
            return midpoint
        if m_squared > x:
            right = midpoint
        else:
            left = midpoint

    return left

if __name__ == "__main__":
    r = 1.0
    sim = Simulation(r)
    _balls = [Ball(
                   pos=(0.2+(i%2)*0.3, 0.2+(i//2)*0.3),
                   vel=(.4 if i == 0 else 0, .3 if i == 0 else 0),
                   r=0.05)
        for i in range(4)]
    sim.add_balls(_balls)
    sim.initialise()
    sim.comic_strip(16)
