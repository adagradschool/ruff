def f():
    try:
        raise ValueError()
    except ValueError:
        print("Caught an error.")
        raise
    else:
        print("No error occurred.")
    finally:
        print("Finally block executed.")