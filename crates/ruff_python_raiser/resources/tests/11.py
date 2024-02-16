def loop_exception(x):
    for i in range(x):
        if i > 5:
            raise Exception("i is too large.")
