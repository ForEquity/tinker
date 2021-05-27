use crate::Order::{OrderHandle, Shares};
use crate::StupidPool::StupidReuse;

struct Pending<'a> {
    order:Vec<&'a OrderHandle>,
    execQty:Shares
}

pub struct Book<'a> {
    symbolId:u16,
    buys:Vec<&'a OrderHandle>,
    sells:Vec<&'a OrderHandle>,
    pendingMatch:Vec<Pending<'a>>
}

impl<'a> Book<'a> {
    pub fn new(symbolId:u16 ) -> Book<'a> {
        Book{ symbolId, buys:Vec::new(), sells:Vec::new(), pendingMatch:Vec::new() }
    }

    pub fn crossBook(&mut self, ord:&OrderHandle ) -> &Vec<Pending> {

        self.pendingMatch.clear();

        &self.pendingMatch
    }

    pub fn postOrder(&mut self, ord:&'a StupidReuse<OrderHandle> ) -> () {

        let (t_vec, indexResult) = {
            if ord.isBuy() {
                let indx = self.buys.iter().position(|other| ord.ge( other ) );
                (&mut self.buys, indx )
            } else {
                let indx = self.sells.iter().position(|other| ord.le( other ) );
                (&mut self.sells, indx )
            }
        };

        if let Some(idx) = indexResult {
            t_vec.insert( idx, ord );
        } else {
            t_vec.push( ord );
        }
    }
    pub fn print_book(&self) -> () {
        self.sells.iter().rev().for_each(|b|println!("{:?}", b));
        println!(" --- ");
        self.buys.iter().for_each(|b| println!("{:?}", b) );
    }
}
