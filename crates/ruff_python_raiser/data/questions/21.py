def f():
    try:
        raise ValueError()
    except ValueError:
        print("Caught an error.")
    finally:
        raise TypeError()