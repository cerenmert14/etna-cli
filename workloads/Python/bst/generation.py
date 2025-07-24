from hypothesis import strategies as st
from impl import T, E

@st.composite
def random_strategy(draw):
    key = draw(st.integers())
    value = draw(st.integers())
    left = draw(st.one_of(random_strategy(), st.just(E())))
    right = draw(st.one_of(random_strategy(), st.just(E())))
    return T(k=key, v=value, left=left, right=right)

@st.composite
def bst_strategy(draw, size=10, lo=0, hi=100):
    if size == 0 or lo >= hi:
        return E()

    key = draw(st.integers(min_value=lo, max_value=hi))
    value = draw(st.integers())

    left = draw(st.one_of(bst_strategy(size-1, lo, key-1), st.just(E())))
    right = draw(st.one_of(bst_strategy(size-1, key+1, hi), st.just(E())))
    return T(k=key, v=value, left=left, right=right)

if __name__ == "__main__":
    # Example usage of the strategies
    from hypothesis import given

    @given(random_strategy())
    def test_random_tree(tree):
        print(tree)
        assert isinstance(tree, (T, E))

    @given(bst_strategy(size=5))
    def test_bst_tree(tree):
        print(tree)
        assert isinstance(tree, (T, E))

    test_random_tree()
    test_bst_tree()