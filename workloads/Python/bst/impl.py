
from dataclasses import dataclass


@dataclass
class T:
    k: int
    v: int
    left: 'BST' = None
    right: 'BST' = None

    def __repr__(self):
        return f"(T {self.left} {self.k} {self.v} {self.right})"


@dataclass
class E:
    def __repr__(self):
        return "(E)"


type BST = T | E
