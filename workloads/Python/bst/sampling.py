from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy
from dataclasses import dataclass
from typing import Callable, Any

from generation import bst_strategy
from spec import (
    prop_delete_delete,
    prop_delete_insert,
    prop_delete_model,
    prop_delete_post,
    prop_delete_union,
    prop_insert_delete,
    prop_insert_insert,
    prop_insert_model,
    prop_insert_post,
    prop_insert_union,
    prop_union_delete_insert,
    prop_union_model,
    prop_union_post,
    prop_union_union_assoc,
    prop_union_union_idempotent,
    prop_delete_valid,
    prop_insert_valid,
    prop_union_valid,
)


# Tree generator similar to Rust Arbitrary (BST built via inserts)
def bespoke_tree() -> SearchStrategy:
    return bst_strategy()


@dataclass
class Property:
    strategy: SearchStrategy
    property: Callable[[Any], bool] | None = None


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


def pretty_tuples(*strategies: SearchStrategy) -> SearchStrategy:
    return st.tuples(*strategies).map(lambda values: Tuple(*values))


def i32() -> SearchStrategy:
    return st.integers(min_value=-(2**31), max_value=2**31 - 1)


# Validity Properties
sample_insert_valid = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32()),
    property=lambda args: prop_insert_valid(*args),
)

sample_delete_valid = Property(
    strategy=pretty_tuples(bespoke_tree(), i32()),
    property=lambda args: prop_delete_valid(*args),
)

sample_union_valid = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree()),
    property=lambda args: prop_union_valid(*args),
)

# Post-condition Properties
sample_insert_post = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32(), i32()),
    property=lambda args: prop_insert_post(*args),
)

sample_delete_post = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32()),
    property=lambda args: prop_delete_post(*args),
)

sample_union_post = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree(), i32()),
    property=lambda args: prop_union_post(*args),
)

# Model-based Properties
sample_insert_model = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32()),
    property=lambda args: prop_insert_model(*args),
)

sample_delete_model = Property(
    strategy=pretty_tuples(bespoke_tree(), i32()),
    property=lambda args: prop_delete_model(*args),
)

sample_union_model = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree()),
    property=lambda args: prop_union_model(*args),
)

# Metamorphic Properties
sample_insert_insert = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32(), i32(), i32()),
    property=lambda args: prop_insert_insert(*args),
)

sample_insert_delete = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32(), i32()),
    property=lambda args: prop_insert_delete(*args),
)

sample_insert_union = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree(), i32(), i32()),
    property=lambda args: prop_insert_union(*args),
)

sample_delete_insert = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32(), i32()),
    property=lambda args: prop_delete_insert(*args),
)

sample_delete_delete = Property(
    strategy=pretty_tuples(bespoke_tree(), i32(), i32()),
    property=lambda args: prop_delete_delete(*args),
)

sample_delete_union = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree(), i32()),
    property=lambda args: prop_delete_union(*args),
)

sample_union_delete_insert = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree(), i32(), i32()),
    property=lambda args: prop_union_delete_insert(*args),
)

sample_union_union_idempotent = Property(
    strategy=bespoke_tree(),
    property=lambda t: prop_union_union_idempotent(t),
)

sample_union_union_assoc = Property(
    strategy=pretty_tuples(bespoke_tree(), bespoke_tree(), bespoke_tree()),
    property=lambda args: prop_union_union_assoc(*args),
)
