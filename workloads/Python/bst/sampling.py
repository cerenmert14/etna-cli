from hypothesis import strategies as st
from generation import bst_strategy
from dataclasses import dataclass
from typing import Callable, Any
from hypothesis.strategies import SearchStrategy

# Replace this with your actual bespoke tree generator
def bespoke_tree():
    return bst_strategy()

@dataclass
class Property:
    strategy: SearchStrategy
    property: Callable[[Any], bool]


class Tuple:
    def __init__(self, *items):
        self.items = items

    def __iter__(self):
        return iter(self.items)

    def __getitem__(self, i):
        return self.items[i]

    def __len__(self):
        return len(self.items)

    def __repr__(self):
        return f"({' '.join(map(str, self.items))})"

    def to_json(self):
        return list(self.items)

def pretty_tuples(*strategies):
    return st.tuples(*strategies).map(lambda values: Tuple(*values))

# Validity Properties
def insert_property(args):
    t, k, v = args
    # your real property check goes here
    return True  # dummy always-passes

sample_insert_valid = Property(
    strategy=pretty_tuples(bespoke_tree(), st.integers(0, 100), st.integers(0, 100)),
    property=insert_property,
)


sample_delete_valid = pretty_tuples(bespoke_tree(), st.integers(min_value=0))
sample_union_valid  = pretty_tuples(bespoke_tree(), bespoke_tree())

# Post-condition Properties
sample_insert_post = pretty_tuples(bespoke_tree(), st.integers(min_value=0), st.integers(min_value=0), st.integers(min_value=0))
sample_delete_post = pretty_tuples(bespoke_tree(), st.integers(min_value=0), st.integers(min_value=0))
sample_union_post  = pretty_tuples(bespoke_tree(), bespoke_tree(), st.integers(min_value=0))

# Model-based Properties
sample_insert_model = sample_insert_valid
sample_delete_model = sample_delete_valid
sample_union_model  = sample_union_valid

# Metamorphic Properties
sample_insert_insert = pretty_tuples(
    bespoke_tree(),
    st.integers(min_value=0), st.integers(min_value=0),
    st.integers(min_value=0), st.integers(min_value=0),
)

sample_insert_delete = pretty_tuples(
    bespoke_tree(),
    st.integers(min_value=0), st.integers(min_value=0),
    st.integers(min_value=0)
)

sample_insert_union = pretty_tuples(
    bespoke_tree(), bespoke_tree(),
    st.integers(min_value=0), st.integers(min_value=0)
)

sample_delete_insert = sample_insert_delete
sample_delete_delete = sample_delete_post
sample_delete_union  = sample_union_post

sample_union_delete_insert = sample_insert_union
sample_union_union_idem = bespoke_tree()
sample_union_union_assoc = pretty_tuples(bespoke_tree(), bespoke_tree(), bespoke_tree())
