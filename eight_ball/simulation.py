import matplotlib.pyplot as plt
from eight_ball import _Simulation
# from eight_ball.ball import Ball
from eight_ball.prelude import draw, floor_sqrt, ceildiv
# from pickle import dump

class Simulation(_Simulation):
    def __init__(self, radius: float):
        self.container_radius = radius

    def comic_strip(self, collisions: int) -> plt.Figure:
        """
        Produce a comic strip of `collisions` collisions within the
        simulation, with the time and velocities of each ball demonstrated.
        """
        width = floor_sqrt(collisions)
        height = ceildiv(collisions, width)
        fig, axs = plt.subplots(height, width)
        draw(axs[0][0], self.get_balls(), self.container_radius, self.global_time)
        for c in range(1, collisions):
            self.next_collision()
            i = c // width
            j = c % width
            draw(axs[i][j], self.get_balls(), self.container_radius, self.global_time)

        fig.tight_layout()
        return fig


# if __name__ == "__main__":
#     sim = Simulation(1.0)
#     _balls = [Ball(
#                  pos=(-0.6+(i%40)*0.03, -0.6+(i//40)*0.03),
#                  vel=(4 if i == 0 else 0, 3 if i == 0 else 0),
#                  r=0.001)
#             for i in range(1600)]
#     sim.add_balls(_balls)
#     sim.initialise()
#     # sim.comic_strip(4)
#     for _ in range(100_000):
#         sim.next_collision()
#     times_dist = sim.collision_times(1_000_000, 0., 0.03, 6000)
#     with open("data/collision_time.pkl", "wb+") as f:
#         dump(times_dist, f)
#     fig = plt.figure()
#     ax = plt.axes()
#     ax.bar(times_dist["centres"], times_dist["counts"], times_dist["width"])
#     plt.show()
