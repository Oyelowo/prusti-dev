#[derive(Clone)]
struct PeerList<T> {
    thing: T
}

#[prusti_contracts::extern_spec]
impl<T: Clone> Clone for PeerList<T> {
    fn clone(&self) -> PeerList<T>;
}

fn main(){}
