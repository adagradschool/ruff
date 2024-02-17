def might_throw():
    raise IndexError("List index out of range.")

def catch_exception():
    try:
        might_throw()
    except IndexError:
        print("Caught an IndexError.")
