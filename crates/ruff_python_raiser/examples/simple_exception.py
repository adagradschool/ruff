def a():
    raise ValueError

def b():
    a()
    try:
        assert True
    except ValueError:
        print("caught")

def c():
    try:
        a()
    except ValueError:
        print("caught")