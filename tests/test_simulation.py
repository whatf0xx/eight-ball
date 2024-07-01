from eight_ball.simulation import floor_sqrt

def test_floor_sqrt():
    assert floor_sqrt(9) == 3
    assert floor_sqrt(8) == 2
    assert floor_sqrt(1) == 1
    assert floor_sqrt(16) == 4
    assert floor_sqrt(17) == 4
    assert floor_sqrt(6) == 2
