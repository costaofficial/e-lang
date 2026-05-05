"""
E — Runtime context
"""


class RuntimeContext:
    def __init__(self):
        self.current_element = None   # set by find (single element)
        self.current_object = None    # set by with
        self.current_item = None      # generic current item (can be list from find all)
        self.current_number = None    # numeric value from get number
        self.current_count = 0        # count from find all
        self.stop_flag = False
        self.object_stack = []

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
