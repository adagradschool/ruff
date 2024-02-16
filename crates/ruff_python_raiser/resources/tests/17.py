class BaseClass:
    def do_something(self):
        raise NotImplementedError("Subclass must implement this method.")

class SubClass(BaseClass):
    def do_something(self):
        raise Exception("Exception in subclass implementation.")

# Usage:
# obj = SubClass()
# obj.do_something()  # This will raise Exception
