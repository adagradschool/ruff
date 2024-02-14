use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Exception {
    name: String,
    children: Vec<Exception>,
}

impl Exception {
    // Constructor function to create a new Exception with no children
    fn new(name: &str) -> Self {
        Exception {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    // Function to add a child exception
    pub fn add_child(&mut self, child: &Exception) {
        self.children.push(child.clone());
    }

    // Recursive function to check if `curr` is a subclass of `base`
    pub fn is_superclass_of(&self, base: &Exception) -> bool {
        if self == base {
            return true;
        }
        for child in &self.children {
            if child.is_superclass_of(base) {
                return true;
            }
        }
        false
    }
}

pub fn get_builtins() -> HashMap<String, Exception>{
    let mut builtins = HashMap::new();
    // Root of the exception tree
    let mut root = Exception::new("Exception");
    // Direct children of Exception
    let system_exit = Exception::new("SystemExit");
    let stop_iteration = Exception::new("StopIteration");
    let mut standard_error = Exception::new("StandardError");
    let mut warning = Exception::new("Warning");


    // Subclasses of StandardError
    let keyboard_interrupt = Exception::new("KeyboardInterrupt");
    let import_error = Exception::new("ImportError");
    let mut environment_error = Exception::new("EnvironmentError");
    let eof_error = Exception::new("EOFError");
    let mut runtime_error = Exception::new("RuntimeError");
    let mut name_error = Exception::new("NameError");
    let attribute_error = Exception::new("AttributeError");
    let mut syntax_error = Exception::new("SyntaxError");
    let type_error = Exception::new("TypeError");
    let assertion_error = Exception::new("AssertionError");
    let mut lookup_error = Exception::new("LookupError");
    let mut arithmetic_error = Exception::new("ArithmeticError");
    let mut value_error = Exception::new("ValueError");
    let mut reference_error = Exception::new("ReferenceError");
    let mut system_error = Exception::new("SystemError");
    let mut memory_error = Exception::new("MemoryError");


    // Subclasses of EnvironmentError
    let mut io_error = Exception::new("IOError");
    let mut os_error = Exception::new("OSError");

    // Adding WindowsError as a child of OSError
    let mut windows_error = Exception::new("WindowsError");
    os_error.add_child(&windows_error);
    environment_error.add_child(&io_error);
    environment_error.add_child(&os_error);

    // Subclasses of RuntimeError
    let mut not_implemented_error = Exception::new("NotImplementedError");
    runtime_error.add_child(&not_implemented_error);

    // Subclasses of NameError
    let mut unbound_local_error = Exception::new("UnboundLocalError");
    name_error.add_child(&unbound_local_error);

    // Subclasses of SyntaxError
    let mut indentation_error = Exception::new("IndentationError");
    indentation_error.add_child(&Exception::new("TabError"));
    syntax_error.add_child(&indentation_error);

    // Subclasses of LookupError
    lookup_error.add_child(&Exception::new("IndexError"));
    lookup_error.add_child(&Exception::new("KeyError"));

    // Subclasses of ArithmeticError
    arithmetic_error.add_child(&Exception::new("OverflowError"));
    arithmetic_error.add_child(&Exception::new("ZeroDivisionError"));
    arithmetic_error.add_child(&Exception::new("FloatingPointError"));

    // Subclasses of ValueError
    let mut unicode_error = Exception::new("UnicodeError");
    unicode_error.add_child(&Exception::new("UnicodeEncodeError"));
    unicode_error.add_child(&Exception::new("UnicodeDecodeError"));
    unicode_error.add_child(&Exception::new("UnicodeTranslateError"));
    value_error.add_child(&unicode_error);

    // Adding children to StandardError
    standard_error.add_child(&keyboard_interrupt);
    standard_error.add_child(&import_error);
    standard_error.add_child(&environment_error);
    standard_error.add_child(&eof_error);
    standard_error.add_child(&runtime_error);
    standard_error.add_child(&name_error);
    standard_error.add_child(&attribute_error);
    standard_error.add_child(&syntax_error);
    standard_error.add_child(&type_error);
    standard_error.add_child(&assertion_error);
    standard_error.add_child(&lookup_error);
    standard_error.add_child(&arithmetic_error);
    standard_error.add_child(&value_error);
    standard_error.add_child(&reference_error);
    standard_error.add_child(&system_error);
    standard_error.add_child(&memory_error);

    // Subclasses of Warning
    warning.add_child(&Exception::new("UserWarning"));
    warning.add_child(&Exception::new("DeprecationWarning"));
    warning.add_child(&Exception::new("PendingDeprecationWarning"));
    warning.add_child(&Exception::new("SyntaxWarning"));
    warning.add_child(&Exception::new("OverflowWarning"));
    warning.add_child(&Exception::new("RuntimeWarning"));
    warning.add_child(&Exception::new("FutureWarning"));

    // Adding direct children to Exception
    root.add_child(&system_exit);
    root.add_child(&stop_iteration);
    root.add_child(&standard_error);
    root.add_child(&warning);

    builtins.insert("Exception".to_owned(), root);
    builtins.insert("KeyboardInterrupt".to_owned(), keyboard_interrupt);
    builtins.insert("ImportError".to_owned(), import_error);
    builtins.insert("EnvironmentError".to_owned(), environment_error);
    builtins.insert("EOFError".to_owned(), eof_error);
    builtins.insert("RuntimeError".to_owned(), runtime_error);
    builtins.insert("NameError".to_owned(), name_error);
    builtins.insert("AttributeError".to_owned(), attribute_error);
    builtins.insert("SyntaxError".to_owned(), syntax_error);
    builtins.insert("TypeError".to_owned(), type_error);
    builtins.insert("AssertionError".to_owned(), assertion_error);
    builtins.insert("LookupError".to_owned(), lookup_error);
    builtins.insert("ArithmeticError".to_owned(), arithmetic_error);
    builtins.insert("ValueError".to_owned(), value_error);
    builtins.insert("ReferenceError".to_owned(), reference_error);
    builtins.insert("SystemError".to_owned(), system_error);
    builtins.insert("MemoryError".to_owned(), memory_error);
    builtins.insert("SystemExit".to_owned(), system_exit);
    builtins.insert("StopIteration".to_owned(), stop_iteration);
    builtins.insert("StandardError".to_owned(), standard_error);
    builtins.insert("Warning".to_owned(), warning);
    builtins.insert("IOError".to_owned(), io_error);
    builtins.insert("OSError".to_owned(), os_error);
    builtins.insert("WindowsError".to_owned(), windows_error);
    builtins.insert("NotImplementedError".to_owned(), not_implemented_error);
    builtins.insert("UnboundLocalError".to_owned(), unbound_local_error);
    builtins.insert("IndentationError".to_owned(), indentation_error);
    builtins.insert("IndexError".to_owned(), Exception::new("IndexError"));
    builtins.insert("KeyError".to_owned(), Exception::new("KeyError"));
    builtins.insert("OverflowError".to_owned(), Exception::new("OverflowError"));
    builtins.insert("ZeroDivisionError".to_owned(), Exception::new("ZeroDivisionError"));
    builtins.insert("FloatingPointError".to_owned(), Exception::new("FloatingPointError"));
    builtins.insert("UnicodeError".to_owned(), unicode_error);
    builtins.insert("UnicodeEncodeError".to_owned(), Exception::new("UnicodeEncodeError"));
    builtins.insert("UnicodeDecodeError".to_owned(), Exception::new("UnicodeDecodeError"));
    builtins.insert("UnicodeTranslateError".to_owned(), Exception::new("UnicodeTranslateError"));
    builtins.insert("UserWarning".to_owned(), Exception::new("UserWarning"));
    builtins.insert("DeprecationWarning".to_owned(), Exception::new("DeprecationWarning"));
    builtins.insert("PendingDeprecationWarning".to_owned(), Exception::new("PendingDeprecationWarning"));
    builtins.insert("SyntaxWarning".to_owned(), Exception::new("SyntaxWarning"));
    builtins.insert("OverflowWarning".to_owned(), Exception::new("OverflowWarning"));
    builtins.insert("RuntimeWarning".to_owned(), Exception::new("RuntimeWarning"));
    builtins.insert("FutureWarning".to_owned(), Exception::new("FutureWarning"));
    builtins
}
