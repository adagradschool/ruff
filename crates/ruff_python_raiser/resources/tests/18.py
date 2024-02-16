def f():
    try:
        raise ValueError()
    except (ValueError, IndexError):
        print("Caught an error.")