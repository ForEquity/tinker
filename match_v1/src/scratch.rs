use indextree::Arena;

fn main() {


    #[derive(Debug,Ord, PartialOrd, Eq, PartialEq)]
    struct Thing { some_str:String }

    // let thing_back = Arena::new();
    // let r = thing_back.alloc( Thing { some_str:"some string".to_string() } );

    let thing_tree = &mut Arena::new();

    let a = thing_tree.new_node( Thing{ some_str:"aya".to_string() });
    

}