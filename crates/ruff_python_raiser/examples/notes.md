- All we care about are functions and class methods.
- For each function, we need to identify each uniquely, (by getting the complete node id from the semantic model?)
- For each function body
    - Add all the raise methods inside the body to the list of exceptions it can throw
    - For try catch block
        Analyze all raises within the body of try
        Go over all exception handler and remove them from naked raises
        Add all the raises from finally block if it exists
        Add all the raises to the parent function's naked raises
    - For all function call in body, get their corresponding raises
    - For function definition, analyze it separately. If it is not called, we don't need to add its raises to the parent.
    - Deal with function calls within other operations (x = someCall())
