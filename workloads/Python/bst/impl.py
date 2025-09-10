from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Tuple, List


@dataclass(eq=True)
class T:
    k: int
    v: int
    left: "BST" | None = None
    right: "BST" | None = None

    def __repr__(self) -> str:  # stable, Rust-like
        return f"(T {self.left} {self.k} {self.v} {self.right})"


@dataclass(eq=True)
class E:
    def __repr__(self) -> str:
        return "(E)"


BST = T | E


FUEL = 10_000


def _node(left: BST | None, k: int, v: int, right: BST | None) -> T:
    return T(k, v, left or E(), right or E())


# Insert
def insert(k: int, v: int, t: BST) -> BST:
    match t:
        case E():
            return _node(E(), k, v, E())
        case T(k2, v2, l, r):
            """! insert """
            if k < k2:
                return _node(insert(k, v, l), k2, v2, r)
            elif k2 < k:
                return _node(l, k2, v2, insert(k, v, r))
            else:
                return _node(l, k2, v, r)
            """!! insert_1 """
            """!
            return _node(E(), k, v, E())
            """
            """!! insert_2 """
            """!
            if k < k2:
                return _node(insert(k, v, l), k2, v2, r)
            else:
                return _node(l, k2, v, r)
            """
            """!! insert_3 """
            """!
            if k < k2:
                return _node(insert(k, v, l), k2, v2, r)
            elif k2 < k:
                return _node(l, k2, v2, insert(k, v, r))
            else:
                return _node(l, k2, v2, r)
            """
            """ !"""


# Join
def join(l: BST, r: BST) -> BST:
    match (l, r):
        case (E(), _):
            return r
        case (_, E()):
            return l
        case (T(k1, v1, l1, r1), T(k2, v2, l2, r2)):
            return _node(l1, k1, v1, _node(join(r1, l2), k2, v2, r2))


# Delete
def delete(k: int, t: BST) -> BST:
    match t:
        case E():
            return E()
        case T(k2, v2, l, r):
            """! delete """
            if k < k2:
                return _node(delete(k, l), k2, v2, r)
            elif k2 < k:
                return _node(l, k2, v2, delete(k, r))
            else:
                return join(l, r)
            """!! delete_4 """
            """!
            _ = v2
            if k < k2:
                return delete(k, l)
            elif k2 < k:
                return delete(k, r)
            else:
                return join(l, r)
            """
            """!! delete_5 """
            """!
            if k2 < k:
                return _node(delete(k, l), k2, v2, r)
            elif k < k2:
                return _node(l, k2, v2, delete(k, r))
            else:
                return join(l, r)
            """
            """ !"""


# Below
def below(k: int, t: BST) -> BST:
    match t:
        case E():
            return E()
        case T(k2, v2, l, r):
            if k <= k2:
                return below(k, l)
            else:
                return _node(l, k2, v2, below(k, r))


# Above
def above(k: int, t: BST) -> BST:
    match t:
        case E():
            return E()
        case T(k2, v2, l, r):
            if k2 <= k:
                return above(k, r)
            else:
                return _node(above(k, l), k2, v2, r)


# Union with fuel (mirrors Rust union_8 variant)
def union_(l: BST, r: BST, f: int) -> BST:
    if f == 0:
        return E()
    f1 = f - 1
    match (l, r):
        case (E(), r):
            return r
        case (l, E()):
            return l
        case (T(k1, v1, l1, r1), T(k2, v2, l2, r2)):
            """! union """
            if k1 == k2:
                return _node(union_(l1, l2, f1), k1, v1, union_(r1, r2, f1))
            elif k1 < k2:
                return _node(
                    union_(l1, below(k1, l2), f1),
                    k1,
                    v1,
                    union_(r1, _node(above(k1, l2), k2, v2, r2), f1),
                )
            else:
                return union_(_node(l2, k2, v2, r2), _node(l1, k1, v1, r1), f1)
            """!! union_6 """
            """!
            return _node(l1, k1, v1, _node(union_(r1, l2, f1), k2, v2, r2))
            """
            """!! union_7 """
            """!
            if k1 == k2:
                return _node(union_(l1, l2, f1), k1, v1, union_(r1, r2, f1))
            elif k1 < k2:
                return _node(l1, k1, v1, _node(union_(r1, l2, f1), k2, v2, r2))
            else:
                return union_(_node(l2, k2, v2, r2), _node(l1, k1, v1, r1), f1)
            """
            """!! union_8 """
            """!
            if k1 == k2:
                return _node(union_(l1, l2, f1), k1, v1, union_(r1, r2, f1))
            elif k1 < k2:
                return _node(
                    union_(l1, below(k1, l2), f1),
                    k1,
                    v1,
                    union_(r1, _node(above(k1, l2), k2, v2, r2), f1),
                )
            else:
                return union_(_node(l2, k2, v2, r2), _node(l1, k1, v1, r1), f1)
            """
            """ !"""


def union(l: BST, r: BST) -> BST:
    return union_(l, r, FUEL)


def find(k: int, t: BST) -> Optional[int]:
    match t:
        case E():
            return None
        case T(k2, v2, l, r):
            if k < k2:
                return find(k, l)
            elif k2 < k:
                return find(k, r)
            else:
                return v2


def size(t: BST) -> int:
    match t:
        case E():
            return 0
        case T(_, _, l, r):
            return 1 + size(l) + size(r)


# Helpers used by specs
def to_list(t: BST) -> List[Tuple[int, int]]:
    match t:
        case E():
            return []
        case T(k, v, l, r):
            return [*to_list(l), (k, v), *to_list(r)]
