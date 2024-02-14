def f():
    raise ValueError("This is a value error")

def complex_expression(x):
    y = x + f()
    return 1 + y
