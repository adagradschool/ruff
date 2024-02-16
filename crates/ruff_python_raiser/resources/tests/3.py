def throws_one_of_two(x):
    if x < 0:
        raise ValueError("Negative value provided.")
    elif x > 100:
        raise OverflowError("Value too large.")
