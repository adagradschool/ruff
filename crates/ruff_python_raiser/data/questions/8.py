class CustomException(Exception):
    pass

def throws_custom_exception():
    raise CustomException("This is a custom exception.")
