from typing import List, Tuple
import matplotlib.pyplot as plt
from matplotlib import patches
from numpy import tanh
from eight_ball.ball import Ball

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

