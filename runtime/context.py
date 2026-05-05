"""
E Runtime Context
"""


class RuntimeContext:
    def __init__(self):
        self.current_element = None
        self.current_object = None
        self.browser = None
        self.stop_flag = False
        self.object_stack = []
        self._browser_count = 0

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
