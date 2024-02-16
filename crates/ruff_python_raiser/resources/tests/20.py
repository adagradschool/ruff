def f():
    try:
        raise ValueError()
    except Exception:
        print("Caught an error.")