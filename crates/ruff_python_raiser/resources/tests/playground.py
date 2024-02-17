import random

def question():
    try:
        return random.choice([1,2,3])
    except IndexError:
        return "Caught an IndexError."