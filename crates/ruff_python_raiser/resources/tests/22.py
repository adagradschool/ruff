class gError(Exception):
    pass

class hError(Exception):
    pass

class jError(Exception):
    pass

class TooLowError(Exception):
    pass

class WrappedHError(Exception):
    pass

def g():
    raise gError("gError")

def h():
    raise hError("hError")

def f():
    _x = 5 + g()
    if _x > 10:
        raise jError("jError")
    try:
        y = 2 + h()
        if y > 10:
            raise TooLowError
    except hError:
        print("Caught hError")
        raise WrappedHError
    except ValueError:
        print("Should not reach here.")
        raise
