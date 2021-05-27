
use fixed::types::I24F40;
use fixed_macro::fixed;
use std::ops::Neg;
use std::cmp::Ordering;
use crate::StupidPool::{StupidPool, StupidReuse, StupidReset};
use once_cell::sync::OnceCell;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::rc::Rc;

pub type Price=I24F40;
pub type Shares=I24F40;
pub const DeMininmis:I24F40 = fixed!(0.0001: I24F40 );
pub const Zero:I24F40 = fixed!(0: I24F40 );


static mut ORDER_SOTRE:Vec<Order> = Vec::new();

// gotta wrap this in a OnceCell cuz no static constructors. and I
// I tired to add one to my StupidPool but I started getting all kinds
// of errors about nightly builds and un-stable code... I just did this.
// aparently, Vec has a const constructor... so it works.
static mut FREE_HANDLE_POOL:OnceCell<StupidPool<OrderHandle>> = OnceCell::new();


pub fn init_order_pool(pool_size:u64) {
    if ! unsafe{ ORDER_SOTRE.is_empty() } {
        println!("tyring to init twice (or pool already created... ignoring");
        return
    }
    unsafe {
        ORDER_SOTRE.reserve_exact(pool_size as usize);
        for x in (0..pool_size) {
            ORDER_SOTRE.push( Order::new(x));
        }

        assert!(std::mem::size_of::<u64>() <= std::mem::size_of::<usize>(), "dunno what kind of janky machine your runnign on here bub.. but usize must be >= u64");

        assert!(FREE_HANDLE_POOL.get().is_none(), "I expect nothing in the pool at this point..");
        FREE_HANDLE_POOL.set( {
            StupidPool::new( pool_size as usize, |x| OrderHandle{ locate:x} )
        });
    }
}

// the handle pool has has this nice Reusable type that will
// just return the handle to the pool when it falls out of
// scope. neato!
pub fn get_free_order() -> Option<StupidReuse<'static, OrderHandle>> {
    let ret = unsafe { FREE_HANDLE_POOL.get().unwrap().try_pull() };
    // deal with a empty pool later...
    // coule return order locate 0 and make sure it's a IOC.. but
    // maybe we can figure out something w/ the type system later
    // to make it less janky
    ret
}

// TODO... return a order type that will only take IOCs.
//pub fn get_last_order -> () {}


#[derive(Debug, PartialEq)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum TIF {
    IOC,
    GTC,
}

#[derive(Debug)]
struct Order {
    locate: u64,
    price: Price,
    symbolId: u16,
    size: Shares,
    side : Side,
    tif : TIF,
    execedQty: Shares
}

impl Order {
    fn new(locate:u64) -> Order {
        Order { locate, price:Zero, symbolId:0, size:Zero, side:Side::BUY, execedQty:Zero, tif:TIF::IOC}
    }
    fn clear(&mut self) -> () {
        self.price = Zero;
        self.symbolId = 0;
        self.size = Zero;
        self.side = Side::BUY;
        self.execedQty = Zero;
    }
}

// our handle. Can pass these around
// and copy them as much as we like.
// use a backing store (vec) for the
// actual data. just go look it up.
// like window 3.0. What's old is new again.
pub struct OrderHandle {
    locate:usize,
}

impl OrderHandle {

    fn new(locate:u64) -> OrderHandle {
        let u = locate as usize;
        let ord = OrderHandle { locate:u };
        ord.clear();
        ord
    }
    #[inline]
    pub fn setNew(&self, price:Price, symboldId:u16, size:Shares, side:Side, tif:TIF ) {

        let v = unsafe { &mut ORDER_SOTRE[self.locate] };
        v.price = price;
        v.symbolId = symboldId;
        v.size = size;
        v.side = side;
        v.execedQty = Zero;
        v.tif = tif;
    }

    #[inline]
    pub fn clear(&self) -> () {
        let v = unsafe { &mut ORDER_SOTRE[self.locate] };
        v.price = Zero;
        v.symbolId = 0;
        v.size = Zero;
        v.side = Side::BUY;
        v.tif = TIF::IOC;
        v.execedQty = Zero;
    }
    // these return values are all the same size, or smaller than a reference (x86/64)
    // so just return by value.
    #[inline]
    pub fn price(&self) -> Price { unsafe{ ORDER_SOTRE[self.locate].price } }
    #[inline]
    pub fn symbolId(&self) -> u16 { unsafe{ ORDER_SOTRE[self.locate].symbolId } }
    #[inline]
    pub fn size(&self) -> Shares { unsafe{ ORDER_SOTRE[self.locate].size } }
    #[inline]
    pub fn tif(&self) -> TIF { unsafe{ ORDER_SOTRE[self.locate].tif } }
    #[inline]
    pub fn isDone(&self) -> bool {
        let o = unsafe{ &ORDER_SOTRE[self.locate] };
        (o.size - o.execedQty) < DeMininmis
    }
    #[inline]
    pub fn exedShares(&self) -> Shares { unsafe{ ORDER_SOTRE[self.locate].execedQty } }
    #[inline]
    pub fn isBuy(&self) -> bool { unsafe{ ORDER_SOTRE[self.locate].side == Side::BUY } }
    #[inline]
    pub fn addExecution(&self, execSize:Shares ) -> () {
        unsafe { let ord = &mut ORDER_SOTRE[self.locate];
            ord.execedQty += execSize;
        }
    }
}

impl fmt::Debug for OrderHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe { ORDER_SOTRE[self.locate].fmt( f ) }
    }
}
// used to clear before it goes back
// into the pool.
impl StupidReset for OrderHandle {
    fn reset(&self) {
        self.clear();
    }
}

impl PartialEq for OrderHandle {
    fn eq(&self, other: &Self) -> bool {
        (self.price() - other.price()).abs().le(&DeMininmis)
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for OrderHandle {}

impl PartialOrd for OrderHandle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp( other ))
    }

}
impl Ord for OrderHandle {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self.price() - other.price();
        if diff > DeMininmis {
            Ordering::Greater
        }else if diff < DeMininmis.neg() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}
