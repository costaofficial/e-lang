"""
E — Runtime context
"""


class Scope:
    def __init__(self, parent=None):
        self.vars = {}
        self.fns = {}  # name -> FnDefinition
        self.parent = parent

    def get_var(self, name: str):
        if name in self.vars:
            return self.vars[name]
        if self.parent:
            return self.parent.get_var(name)
        raise NameError(f"variable '{name}' not defined")

    def set_var(self, name: str, value):
        if name in self.vars:
            self.vars[name] = value
        elif self.parent:
            self.parent.set_var(name, value)
        else:
            self.vars[name] = value

    def def_var(self, name: str, value):
        self.vars[name] = value

    def def_fn(self, name: str, fn_def):
        self.fns[name] = fn_def

    def get_fn(self, name: str):
        if name in self.fns:
            return self.fns[name]
        if self.parent:
            return self.parent.get_fn(name)
        raise NameError(f"function '{name}' not defined")


class RuntimeContext:
    def __init__(self):
        self.current_element = None
        self.current_object = None
        self.current_item = None
        self.current_number = None
        self.current_count = 0
        self.stop_flag = False
        self.object_stack = []
        self.scope = Scope()  # global scope

    def push_scope(self):
        self.scope = Scope(parent=self.scope)

    def pop_scope(self):
        if self.scope.parent:
            self.scope = self.scope.parent

    def push_object(self, obj):
        self.object_stack.append(self.current_object)
        self.current_object = obj

    def pop_object(self):
        self.current_object = self.object_stack.pop() if self.object_stack else None

    @property
    def should_stop(self):
        return self.stop_flag

    def stop(self):
        self.stop_flag = True
