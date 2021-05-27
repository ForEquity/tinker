#![feature(const_fn)]

// use dynamic_pool::DynamicPool;
//use object_pool::Pool;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;


pub trait StupidReset {
    fn reset(&self);
}

pub type Stack<T> = Vec<T>;


pub struct StupidPool<T: StupidReset> {
    objects:RefCell<Stack<T>>,
}

impl<T: StupidReset > StupidPool<T> {
    pub fn new<F>(cap: usize, init: F) -> StupidPool<T>
        where
            F: Fn(usize) -> T,
    {
        let mut objects = Stack::new();

        for x in (0..cap).rev() {
            objects.push(init(x));
        }

        StupidPool {
            objects:RefCell::new(objects)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.objects.borrow().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.objects.borrow().is_empty()
    }

    #[inline]
    pub fn try_pull(&self) -> Option<StupidReuse<T>> {
        self.objects
            .borrow_mut().pop()
            .map(|data| StupidReuse::new(&self, data) )
    }

    // #[inline]
    // pub fn pull<F: Fn() -> T>(&self, fallback: F) -> Reusable<T> {
    //     self.try_pull()
    //         .unwrap_or_else(|| Reusable::new(self, fallback()))
    // }

    #[inline]
    pub fn attach(&self, t: T) {
        self.objects.borrow_mut().push(t)
    }
}

pub struct StupidReuse<'a, T>
    where T:StupidReset
{
    pool: &'a StupidPool<T>,
    data: ManuallyDrop<T>,
}

impl<'a, T> StupidReuse<'a, T>
    where T: StupidReset
{
    #[inline]
    pub fn new(pool: &'a StupidPool<T>, t: T) -> Self {
        Self {
            pool,
            data: ManuallyDrop::new(t),
        }
    }

    // #[inline]
    // pub fn detach(mut self) -> (&'a Pool<T>, T) {
    //     let ret = unsafe { (self.pool, self.take()) };
    //     forget(self);
    //     ret
    // }

    fn take(&mut self) -> T {
        unsafe { ManuallyDrop::take(&mut self.data) }
    }
}

impl<'a, T> Deref for StupidReuse<'a, T>
    where T: StupidReset
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T { &self.data }
}

// impl<'a, T> DerefMut for Reusable<'a, T> {
//     #[inline]
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.data
//     }
// }

impl<T> fmt::Debug for StupidReuse<'_, T>
    where T : fmt::Debug + StupidReset
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T> Drop for StupidReuse<'a, T>
    where
        T : StupidReset
{

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let  ret = self.take();
            // println!("returning to pool {:?}",ret);
            self.data.reset();
            self.pool.attach(ret);
        }
    }
}

// fn do_something() {
//     // let thing = DynamicPool::new();
//     let other = Pool::new();
// }