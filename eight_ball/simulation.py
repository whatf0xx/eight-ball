import matplotlib.pyplot as plt
from eight_ball import _Simulation
from eight_ball.ball import Ball
from eight_ball.prelude import draw, floor_sqrt, ceildiv

class Simulation(_Simulation):
    def __init__(self, radius: float):
        self.container_radius = radius

    def comic_strip(self, collisions: int):
        """
        Produce a comic strip of `collisions` collisions within the
        simulation, with the time and velocities of each ball demonstrated.
        """
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


if __name__ == "__main__":
    sim = Simulation(1.0)
    _balls = [Ball(
                   pos=(0.2+(i%2)*0.3, 0.2+(i//2)*0.3),
                   vel=(.4 if i == 0 else 0, .3 if i == 0 else 0),
                   r=0.05)
        for i in range(4)]
    sim.add_balls(_balls)
    sim.initialise()
    times_dist = sim.collision_times(3_000_000, 0., 7., 1400)
    fig = plt.figure()
    ax = plt.axes()
    ax.bar(times_dist["centres"], times_dist["counts"], times_dist["width"])
    plt.show()
