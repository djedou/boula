mod b_node;
mod node_type;



pub use node_type::*;
pub use b_node::*;


/// The page size is defined to be 4K bytes. A larger page size such as 8K or 16K also works.

/*
type BTree struct {
    
    root uint64
    // callbacks for managing on-disk pages
    get func(uint64) BNode // dereference a pointer
    new func(BNode) uint64 
    del func(uint64)       // deallocate a page
}
*/

pub struct BTree {
    pub root: u64 //[u8; 8], // pointer (a nonzero page number)
}

/*
impl BTree {
    // allocate a new page
    //pub fn new(node: BNode) -> 

    pub fn btype(node BNode) -> {

    }
}
*/