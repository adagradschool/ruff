def might_throw_again():
    raise KeyError("Key not found in dictionary.")

def outside_try_catch():
    try:
        # Some safe operation
        print("This part is safe.")
    except KeyError:
        print("Caught an exception.")
    
    # Calling a function that might throw, outside try-catch
    might_throw_again()
