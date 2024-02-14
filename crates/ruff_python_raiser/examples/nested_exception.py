import random
class ExceptionA(Exception):
    def __init__(self, message):
        self.message = message
    pass

class ExceptionB(Exception):
    pass

class ExceptionC(Exception):
    pass

def c():
    def d():
        if bool(random.getrandbits(1)):
            raise Exception("An error occurred in function d")
    if bool(random.getrandbits(1)):
        raise ExceptionC("An error occurred in function c")

def b():
    if bool(random.getrandbits(1)):
        raise ExceptionB("An error occurred in function b independently of c")
    c()

def a():
    if bool(random.getrandbits(1)):
        raise ExceptionA("An error occurred in function a")
    b()  # a calls b


# Example of calling function a and handling its exceptions
try:
    a()
except ExceptionA as e:
    print(f"Caught in global scope (from a): {e}")
except ExceptionB as e:
    print(f"Caught in global scope (from b): {e}")
except ExceptionC as e:
    # This catch is necessary if c's exception could propagate to this point, which in this setup, it won't.
    print(f"Caught in global scope (from c): {e}")