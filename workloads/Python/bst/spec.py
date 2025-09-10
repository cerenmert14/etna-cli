from __future__ import annotations

from typing import Callable, List, Optional, Tuple

from impl import BST, E, T, insert, delete, union, find, to_list


def keys(t: BST) -> List[int]:
    match t:
        case E():
            return []
        case T(k, _v, l, r):
            return [k, *keys(l), *keys(r)]


def is_bst(t: BST) -> bool:
    match t:
        case E():
            return True
        case T(k, _v, l, r):
            return (
                is_bst(l)
                and is_bst(r)
                and all(k2 < k for k2 in keys(l))
                and all(k2 > k for k2 in keys(r))
            )


def delete_key(k: int, xs: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    return [(x, v) for (x, v) in xs if x != k]


def l_insert(pair: Tuple[int, int], xs: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    k, v = pair
    inserted = False
    result: List[Tuple[int, int]] = []
    for (k2, v2) in xs:
        if not inserted and k < k2:
            result.append((k, v))
            inserted = True
        if k == k2 and not inserted:
            result.append((k, v))
            inserted = True
        else:
            result.append((k2, v2))
    if not inserted:
        result.append((k, v))
    return result


def l_sort(xs: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    result: List[Tuple[int, int]] = []
    for kv in xs:
        result = l_insert(kv, result)
    return result


def l_find(k: int, xs: List[Tuple[int, int]]) -> Optional[int]:
    for (k2, v) in xs:
        if k2 == k:
            return v
    return None


def l_union_by(f: Callable[[int, int], int], l1: List[Tuple[int, int]], l2: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
    result = list(l2)
    for (k, v) in l1:
        result = [(k2, v2) for (k2, v2) in result if k2 != k]
        v2 = l_find(k, l2)
        v_new = f(v, v2) if v2 is not None else v
        result = l_insert((k, v_new), result)
    return result


def implies(cond: bool, th: Callable[[], bool]) -> bool:
    return (not cond) or th()


def prop_insert_valid(t: BST, k: int, v: int) -> bool:
    return implies(is_bst(t), lambda: is_bst(insert(k, v, t)))


def prop_delete_valid(t: BST, k: int) -> bool:
    return implies(is_bst(t), lambda: is_bst(delete(k, t)))


def prop_union_valid(t1: BST, t2: BST) -> bool:
    return implies(is_bst(t1), lambda: is_bst(union(t1, t2)))


def prop_insert_post(t: BST, k: int, k2: int, v: int) -> bool:
    return implies(
        is_bst(t),
        lambda: find(k2, insert(k, v, t)) == (v if k == k2 else find(k2, t)),
    )


def prop_delete_post(t: BST, k: int, k2: int) -> bool:
    return implies(
        is_bst(t),
        lambda: find(k2, delete(k, t)) == (None if k == k2 else find(k2, t)),
    )


def prop_union_post(t1: BST, t2: BST, k: int) -> bool:
    return implies(
        is_bst(t1),
        lambda: implies(
            is_bst(t2),
            lambda: (
                find(k, union(t1, t2))
                == (find(k, t1) if (find(k, t1) is not None) else find(k, t2))
            ),
        ),
    )


def prop_insert_model(t: BST, k: int, v: int) -> bool:
    return implies(
        is_bst(t),
        lambda: to_list(insert(k, v, t)) == l_insert((k, v), delete_key(k, to_list(t))),
    )


def prop_delete_model(t: BST, k: int) -> bool:
    return implies(is_bst(t), lambda: to_list(delete(k, t)) == delete_key(k, to_list(t)))


def prop_union_model(t1: BST, t2: BST) -> bool:
    return implies(
        is_bst(t1),
        lambda: implies(
            is_bst(t2),
            lambda: to_list(union(t1, t2))
            == l_sort(l_union_by(lambda x, _y: x, to_list(t1), to_list(t2))),
        ),
    )


def prop_insert_insert(t: BST, k: int, k2: int, v: int, v2: int) -> bool:
    return implies(
        is_bst(t),
        lambda: (
            insert(k, v, insert(k2, v2, t))
            == (insert(k, v, t) if k == k2 else insert(k2, v2, insert(k, v, t)))
        ),
    )


def prop_insert_delete(t: BST, k: int, k2: int, v: int) -> bool:
    return implies(
        is_bst(t),
        lambda: (
            insert(k, v, delete(k2, t))
            == (insert(k, v, t) if k == k2 else delete(k2, insert(k, v, t)))
        ),
    )


def prop_insert_union(t: BST, t2: BST, k: int, v: int) -> bool:
    return implies(
        is_bst(t),
        lambda: implies(
            is_bst(t2), lambda: insert(k, v, union(t, t2)) == union(insert(k, v, t), t2)
        ),
    )


def prop_delete_insert(t: BST, k: int, k2: int, v: int) -> bool:
    return implies(
        is_bst(t),
        lambda: (
            delete(k, insert(k2, v, t))
            == (delete(k, t) if k == k2 else insert(k2, v, delete(k, t)))
        ),
    )


def prop_delete_delete(t: BST, k: int, k2: int) -> bool:
    return implies(is_bst(t), lambda: delete(k, delete(k2, t)) == delete(k2, delete(k, t)))


def prop_delete_union(t1: BST, t2: BST, k: int) -> bool:
    return implies(
        is_bst(t1),
        lambda: implies(
            is_bst(t2),
            lambda: delete(k, union(t1, t2)) == union(delete(k, t1), delete(k, t2)),
        ),
    )


def prop_union_delete_insert(t1: BST, t2: BST, k: int, v: int) -> bool:
    return implies(
        is_bst(t1),
        lambda: implies(
            is_bst(t2),
            lambda: union(delete(k, t1), insert(k, v, t2))
            == insert(k, v, union(t1, t2)),
        ),
    )


def prop_union_union_idempotent(t: BST) -> bool:
    return implies(is_bst(t), lambda: union(t, t) == t)


def prop_union_union_assoc(t1: BST, t2: BST, t3: BST) -> bool:
    return implies(
        is_bst(t1),
        lambda: implies(
            is_bst(t2),
            lambda: implies(
                is_bst(t3),
                lambda: union(t1, union(t2, t3)) == union(union(t1, t2), t3),
            ),
        ),
    )

