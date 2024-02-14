class MethodCalledByConstructor:
    def __init__(self, value):
        self.validate_value(value)

    def validate_value(self, value):
        if value < 10:
            raise ValueError("Value must be at least 10.")

# Usage:
# obj = MethodCalledByConstructor(5)  # This will raise ValueError
