class ConstructorException:
    def __init__(self, value):
        if value < 0:
            raise ValueError("Negative value not allowed.")

# Usage:
# obj = ConstructorException(-1)  # This will raise ValueError
