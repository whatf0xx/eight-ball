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
        self.ax.set_xlim(-1., 1.)
        self.ax.set_ylim(-1., 1.)
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
        self.container = Container(r=1)
        self.drawer = LabDrawer(**kwargs)
        self.balls = []
        if balls is not None:
            self.add_balls(balls)

        self.drawer.add(self.container.patch)
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
        Given the indices of two `Ball`s in the lab, return the collision
        between them as a `CollisionEvent` tuple. If no collision exists, return
        `None`. The container should only turn up in the second index. In this
        case, it will correspond to `j=n`, `n` being the length of the array, in
        which case the container is used, instead.
        """
        ball = self.balls[i]
        n = len(self.balls)
        if j == n:
            # this is the container
            collision_time = ball.time_to_container_collision(self.container)
            pair_hash = ball.container_hash()
        else:
            other = self.balls[j]
            collision_time = ball.time_to_collision(other)
            pair_hash = ball.pair_hash(other)

        match collision_time:
            case None:
                return None
            case collision_time:
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
            for j in range(i, n):
                maybe_collision = self.calculate_collision_event(i, j)
                self._push_collision_or_pass(maybe_collision)

            maybe_collision = self.calculate_collision_event(i, n)
            self._push_collision_or_pass(maybe_collision)

        maybe_collision = self.calculate_collision_event(n-1, n)
        self._push_collision_or_pass(maybe_collision)

    def _push_collision_or_pass(self, maybe_collision):
        match maybe_collision:
            case None:
                pass
            case collision:
                heapq.heappush(self.collision_queue, collision)

    def calculate_collisions_for_ball(self, index: int):
        """
        For the `Ball` at index `index`, calculate the next collision between it
        and the rest of the lab and then push the collision to the queue.
        """
        n = len(self.balls)
        left = range(index)
        right = range(index+1, n)
        for j in chain(left, right):
            maybe_collision = self.calculate_collision_event(index, j)
            self._push_collision_or_pass(maybe_collision)

        maybe_collision = self.calculate_collision_event(index, n)
        self._push_collision_or_pass(maybe_collision)

    def get_next_collision(self) -> CollisionEvent:
        """
        Because collision events are on a queue, there is a possibility that
        collisions could occur between when a `CollisionEvent` is added and
        when the time comes to execute the collision. So, the `pair_hash` is
        used to ensure that the velocities have not changed in the interim.
        """
        n = len(self.balls)
        while True:
            potential_collision = heapq.heappop(self.collision_queue)
            i, j, pair_hash = potential_collision[1:]  # don't care about the time
            if j == n:
                local_hash = self.balls[i].container_hash()
            else:
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
        n = len(self.balls)
        if i == n:  # corresponds to `i` is the container
            self.balls[j].container_collide(self.container)
            self.calculate_collisions_for_ball(j)
        elif j == n:  # `j` is the container
            self.balls[i].container_collide(self.container)
            self.calculate_collisions_for_ball(i)
        else:
            self.balls[i].collide(self.balls[j])
            self.calculate_collisions_for_ball(i)
            self.calculate_collisions_for_ball(j)

    def step_to_next_collision(self):
        """
        Move the simulation forward to the next collision, perform the
        collision, and update the collision queue with the involved
        `Ball`s' new trajectories. This does NOT update the graphical
        animation, that should be handled at the end of the step.
        """
        time, i, j, _ = self.next_collision  # don't need to check pair_hash
        delta_t = time - self.global_time
        assert delta_t >= 0, f"Collision took place in the past!?\n{time=};\t{self.global_time=}"
        for ball in self.balls:
            ball.step(delta_t)
        self.global_time = time
        self.collide_update_queue(i, j)
        self.next_collision = self.get_next_collision()


    def advance_through_collision(self):
        """
        Advance the simulation, calculating the kinematics of the collision
        event that happens during the step.
        """
        original_step = self.global_time + self.step
        while original_step > self.next_collision[0]:
            # if we just stepped, we would miss a `CollisionEvent`
            self.step_to_next_collision()

        remaining_step = original_step - self.global_time
        for ball, patch in zip(self.balls, self.drawer.ball_patches):
            ball.step(remaining_step)
            patch.set_center(ball.pos)
        self.global_time = original_step

    def run(self, frames=500, interval=20):
        """
        Invoke the `run` method of the contained `LabDrawer` object to
        run the simulation and corresponding animation.
        """
        self.drawer.run(self.update, frames=frames, interval=interval)


if __name__ == "__main__":
    # ball1 = Ball(pos=(-0.29, -0.3), vel=(0.04, 0.04), r=0.05)
    # ball2 = Ball(pos=(0.3, 0.3), vel=(-0.02, -0.02), r=0.05)

    # lab = Lab([ball1, ball2], step=.1)
    # lab.run()
    # plt.show()
    _balls = [Ball(
                   pos=(-0.6+(i%20)*0.06, -0.6+(i//20)*0.06),
                   vel=(4 if i == 0 else 0, 3 if i == 0 else 0),
                   r=0.01)
        for i in range(400)]
    lab = Lab(_balls, step=1e-2)
    lab.run(frames=100000, interval=2)
    plt.show()
