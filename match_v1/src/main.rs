
mod Order;
mod StupidPool;
mod Book;

fn main() {

    println!("hello world");

    Order::init_order_pool( 4096 );


    let mut book = Book::Book::new(16);


    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 50.25 ), 32, Order::Shares::from_num( 100 ), Order::Side::BUY, Order::TIF::GTC );
    book.postOrder( &ord );



    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 50 ), 32, Order::Shares::from_num( 100 ), Order::Side::BUY, Order::TIF::GTC  );
    book.postOrder( &ord );

    println!("price = {}",ord.price());


    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 51 ), 32, Order::Shares::from_num( 100 ), Order::Side::BUY, Order::TIF::GTC  );
    book.postOrder( &ord );

    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 50.25 ), 32, Order::Shares::from_num( 100 ), Order::Side::BUY, Order::TIF::GTC  );
    book.postOrder( &ord );


    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 51 ), 32, Order::Shares::from_num( 100 ), Order::Side::SELL, Order::TIF::GTC  );
    book.postOrder( &ord );

    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 50.25 ), 32, Order::Shares::from_num( 100 ), Order::Side::SELL, Order::TIF::GTC  );
    book.postOrder( &ord );

    let ord = Order::get_free_order().unwrap();
    ord.setNew( Order::Price::from_num( 52 ), 32, Order::Shares::from_num( 100 ), Order::Side::SELL, Order::TIF::GTC  );
    book.postOrder( &ord );

    book.print_book();

}