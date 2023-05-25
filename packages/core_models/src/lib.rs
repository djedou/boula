//pub mod b_node;
pub mod b_tree;



pub const HEADER: usize = 4; // 4B
pub const BTREE_PAGE_SIZE: usize = 4096; // 4096B
pub const BTREE_MAX_KEY_SIZE: usize = 1000; // 1000B
pub const BTREE_MAX_VAL_SIZE: usize = 3000; // 3000B


#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::{assert_le, assert_ge};


    #[test]
    fn less_or_eq() {
        let node_max = HEADER + 8 + 2 + 4 + BTREE_MAX_KEY_SIZE + BTREE_MAX_VAL_SIZE;
        assert_le!(node_max, BTREE_PAGE_SIZE);
    }

    #[test]
    fn great_or_eq() {
        let node_max = HEADER + 8 + 2 + 4 + BTREE_MAX_KEY_SIZE + BTREE_MAX_VAL_SIZE;
        assert_ge!(BTREE_PAGE_SIZE, node_max);
    }

    #[test]
    fn not_eq() {
        let node_max = HEADER + 8 + 2 + 4 + BTREE_MAX_KEY_SIZE + BTREE_MAX_VAL_SIZE;
        assert_ne!(node_max, BTREE_PAGE_SIZE);
    }
}
