class StaticMethodException:
    @staticmethod
    def calculate_division(x, y):
        if y == 0:
            raise ZeroDivisionError("Cannot divide by zero.")

# Usage:
# StaticMethodException.calculate_division(10, 0)  # This will raise ZeroDivisionError
