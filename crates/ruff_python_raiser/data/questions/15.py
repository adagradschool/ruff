class ClassMethodException:
    _threshold = 10

    @classmethod
    def set_threshold(cls, new_threshold):
        if new_threshold < 0:
            raise ValueError("Threshold cannot be negative.")
        cls._threshold = new_threshold

# Usage:
# ClassMethodException.set_threshold(-5)  # This will raise ValueError
