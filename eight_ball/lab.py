"""
Module containing the `Lab` class and associated helper objects, which
facilitates the simulation of billiard-ball kinematics in a more inter-
active and visual manner. For a more performant simulation, see the
`Simulation` module.
"""
from typing import Iterator
import heapq
from itertools import chain
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from .ball import Ball, Container


type CollisionEvent = tuple[float, int, int, float]
"""
A `CollisionEvent` records a collision between two balls within a `Lab`
simulation. The elements of the tuple are stuctured as follows:
- 0: the time at which the collision will take place, as a float;
- 1: the index of the first `Ball` involved in the collision;
- 2: the index of the second `Ball` invoved in the collision;
- 3: the dot product of the `Ball`'s velocities*.    

*This is used to ensure that the `Ball`'s velocities have not changed
since the instanciation of the CollisionEvent.
"""


class LabDrawer:
    """
    A helper class for `Lab` which handles the animation in `matplotlib` and
    the rendering of `Patch` objects.    
    """
    def __init__(self, ball_patches=None, **kwargs):
        self.ball_patches = ball_patches if ball_patches is not None else []
        self.fig = plt.figure(**kwargs)
        self.ax = plt.axes(**kwargs)
        for patch in self.ball_patches:
            self.ax.add_patch(patch)
        self.ani = None

    def add(self, patch):
        """
        Add a `Patch` object to the `LabDrawer` so that it can include it in
        the simulation.
        """
        self.ball_patches.append(patch)
        self.ax.add_patch(patch)

    def run(self, updater, frames=500, interval=20):
        """
        Run the simulation via the `updater` function.
        """
        self.ani = FuncAnimation(self.fig,
                                 updater,
                                 frames=frames,
                                 interval=interval,
                                 repeat=False,
                                 blit=True)


class Lab:
    """
    A simulation of billard-ball dynamics. The underlying calculations
    for the kinematics are handled in Rust, while the book-keeping for
    the simulation is in Python, for ease of understanding and compati-
    bility with `matplotlib`. For a more performant simulation, see the
    `Simulation` class, which handles the book-keeping wholly in Rust.
    """
    def __init__(self, balls=None, step=1e-2, **kwargs):
        container = Container(pos=(0.5, 0.5), vel=(0., 0.), r=-0.5)
        self.balls = [container]
        self.drawer = LabDrawer(**kwargs)
        self.drawer.add(container.patch)
        if balls is not None:
            self.add_balls(balls)

        self.step = step
        self.global_time = 0.

        self.collision_queue = []
        self.calculate_collisions_global()
        self.next_collision = heapq.heappop(self.collision_queue)

    def add_ball(self, ball: Ball):
        """
        Type-checked addition of a `Ball` to the lab. Attempting to add
        an object which is not a `Ball` will result in a `ValueError`.    
        """
        if isinstance(ball, Ball):
            self.balls.append(ball)
            self.drawer.add(ball.patch)
        else:
            raise ValueError("Can only add a `Ball` to the lab.")

    def add_balls(self, balls: Iterator[Ball]):
        """
        Add multiple `Ball`s to the lab via an `Iterator`.
        """
        for ball in balls:
            self.add_ball(ball)

    def calculate_collision_event(self, i: int, j: int) -> CollisionEvent | None:
        """
        Given the indices of two `Ball`s in the lab, return the collision between
        them as a `CollisionEvent` tuple. If no collision exists, return `None`.
        """
        ball = self.balls[i]
        other = self.balls[j]

        collision_time = ball.time_to_collision(other)
        match collision_time:
            case None:
                return None
            case collision_time:
                pair_hash = ball.pair_hash(other)
                return (collision_time + self.global_time, i, j, pair_hash)

    def calculate_collisions_global(self):
        """
        For each `Ball` in the lab, calculate its next collision with
        another ball in the lab and push this to a priority queue stored
        in `self.collision_queue`. For information on how the collisions
        are stored, see the type alias `CollisionEvent`.
        """
        n = len(self.balls)
        for i in range(n-1):
            soonest_collision = (float('inf'), None, None, None)
            for j in range(i, n):
                collision = self.calculate_collision_event(i, j)
                match collision:
                    case None:
                        pass
                    case collision:
                        soonest_collision = min(soonest_collision, collision)
            heapq.heappush(self.collision_queue, soonest_collision)

    def calculate_collisions_for_ball(self, index: int):
        """
        For the `Ball` at index `index`, calculate the next collision between it
        and the rest of the lab and then push the collision to the queue.
        """
        left = range(index)
        right = range(index+1, len(self.balls))
        soonest_collision = (float('inf'), None, None, None)
        for j in chain(left, right):
            collision = self.calculate_collision_event(index, j)
            match collision:
                case None:
                    pass
                case collision:
                    soonest_collision = min(soonest_collision, collision)

        heapq.heappush(self.collision_queue, soonest_collision)

    def get_next_collision(self) -> CollisionEvent:
        """
        Because collision events are on a queue, there is a possibility that
        collisions could occur between when a `CollisionEvent` is added and
        when the time comes to execute the collision. So, the `pair_hash` is
        used to ensure that the velocities have not changed in the interim.
        """
        while True:
            potential_collision = heapq.heappop(self.collision_queue)
            i, j, pair_hash = potential_collision[1:]  # don't care about the time
            local_hash = self.balls[i].pair_hash(self.balls[j])
            if local_hash == pair_hash:
                return potential_collision

    def update(self, _):
        """
        Advance the simulation according to whether or not it is close to a
        collision event.
        """
        if self.next_collision[0] - self.global_time > self.step:
            self.advance()
        else:
            self.advance_through_collision()

        return (patch for patch in self.drawer.ball_patches)

    def advance(self):
        """
        Move the simulation forward by `self.step`, taking care to update
        the `Patch` objects for the corresponding animation.
        """
        self.global_time += self.step
        for ball, patch in zip(self.balls, self.drawer.ball_patches):
            ball.step(self.step)
            patch.set_center(ball.pos)

    def collide_update_queue(self, i, j):
        """
        Perform a collision between `self.balls` `i` and `j` and calculate
        the next collisions for each of them. Carefully handle the case of
        either being the container, separately.
        """
        if i == 0:  # corresponds to `i` is the container
            self.balls[j].container_collide(self.balls[i])
            self.calculate_collisions_for_ball(j)
        elif j == 0:  # `j` is the container
            self.balls[i].container_collide(self.balls[j])
            self.calculate_collisions_for_ball(i)
        else:
            self.balls[i].collide(self.balls[j])
            self.calculate_collisions_for_ball(i)
            self.calculate_collisions_for_ball(j)

    def advance_through_collision(self):
        """
        Advance the simulation, calculating the kinematics of the collision
        event that happens during the step.
        """
        before = self.next_collision[0] - self.global_time
        after = self.step - before

        for ball in self.balls:
            ball.step(before)
        self.global_time += before

        i, j = self.next_collision[1], self.next_collision[2]
        self.collide_update_queue(i, j)
        self.next_collision = self.get_next_collision()

        for ball, patch in zip(self.balls, self.drawer.ball_patches):
            ball.step(after)
            patch.set_center(ball.pos)
        self.global_time += after

    def run(self):
        """
        Invoke the `run` method of the contained `LabDrawer` object to
        run the simulation and corresponding animation.
        """
        self.drawer.run(self.update)


if __name__ == "__main__":
    ball1 = Ball(pos=(0.21, 0.2), vel=(0.04, 0.04), r=0.05)
    ball2 = Ball(pos=(0.8, 0.8), vel=(-0.02, -0.02), r=0.05)

    lab = Lab([ball1, ball2], step=.1)
    lab.run()
    plt.show()
