def input_based_exception(x):
    if not isinstance(x, int):
        raise TypeError("Input must be an integer.")
