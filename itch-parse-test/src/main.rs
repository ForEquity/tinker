use std::env;

use std::fs::File;
use std::path::Path;

use std::io;
use std::io::prelude::*;
use std::io::BufReader;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

//testing and debugs
use std::time::Instant;
const DEBUG : bool = false;
const OUTPUT : bool = true;


struct PriceLevel {
	price: u32, 
	count: u32, 
}

struct Book {
	buys: Vec<PriceLevel>,
	sells: Vec<PriceLevel>
}

struct Order {
	id : u64, 
    stock_name: String,
    side: char, 
    shares: u32,
    price: u32,
}



fn insert_order(book: &mut Book, order: & Order) {
	
	let levels = match order.side {
		'B' => &mut book.buys,
		_ => &mut book.sells,
	};

	let mut pos = 0;

	//WILDLY inefficent

	for mut pl in levels.into_iter() {

		if order.price == pl.price {
			pl.count = pl.count + 1;
			//println!("incrementing pricelevel count to  {}", pl.count);			
			return;
		}

		if order.side == 'B' && order.price > pl.price { break; }
		if order.side == 'S' && order.price < pl.price { break; }		
		pos = pos + 1;
	}

	let pl = PriceLevel {
		price: order.price,
		count: 1,
	};

	//println!("adding level at {} with old len {} with price {}", pos, levels.len(), order.price);
	levels.insert(pos, pl);
}

fn remove_order(book: &mut Book, order: & Order) {
	
	let  levels = match order.side {
		'B' => &mut book.buys,
		_ => &mut book.sells,
	};

	let mut pos = 0;

	//WILDLY inefficent

	for mut pl in levels.into_iter() {

		if order.price == pl.price {
			pl.count = pl.count - 1;

			if pl.count == 0{
				break;
			} else {	
				return;
			}
		}
		pos = pos + 1;
	}
	//println!("removing level at {} with len {} with price {}", pos, levels.len(), order.price);
	levels.remove(pos);
}

fn as_u64_be6(array: &[u8]) -> u64 {
    ((array[0] as u64) << 40) +
    ((array[1] as u64) << 32) +
    ((array[2] as u64) << 24) +
    ((array[3] as u64) << 16) +
    ((array[4] as u64) <<  8) +
    ((array[5] as u64) <<  0)
}

fn as_u64_be8(array: &[u8]) -> u64 {
	((array[0] as u64) << 56) +
    ((array[1] as u64) << 48) +
    ((array[2] as u64) << 40) +
    ((array[3] as u64) << 32) +
    ((array[4] as u64) << 24) +
    ((array[5] as u64) << 16) +
    ((array[6] as u64) <<  8) +
    ((array[7] as u64) <<  0)
}

fn as_u32_be4(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

fn as_u32_be2(array: &[u8]) -> u32 {
    ((array[0] as u32) <<  8) +
    ((array[1] as u32) <<  0)
}


fn process_system_event(data: &[u8]) -> char {

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);

	let ec = data[11] as char;

	if OUTPUT { println!("Event Code:{}", ec);}

	ec
}

fn process_stock_map(data: &[u8], stocks: &mut HashMap<u32, String>) {

	let locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);
	
	let stock_name = String::from_utf8_lossy(&data[11..19]).to_string();
	
	if OUTPUT { println!("Adding {} at {}", stock_name, locate); }

	stocks.insert(locate, stock_name);
	
	//don't care about the rest
	
}

fn process_add_order(data: &[u8], orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, attribution: bool) {

	let _locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);
	
	let id = as_u64_be8(&data[11..19]);
	
	let side = data[19] as char;

	let shares = as_u32_be4(&data[20..24]);

	let stock_name = String::from_utf8_lossy(&data[24..32]).to_string();

	let price = as_u32_be4(&data[32..36]);

	let dprice = price as f32 / 10_000.00;
	if OUTPUT { println!("A {} {} {} {} {}", id, side, shares, stock_name, dprice);} 

	let stock_key = stock_name.clone();

	let book: &mut Book = match books.entry(stock_key) {
   		Entry::Occupied(o) => o.into_mut(),
   		Entry::Vacant(v) => v.insert(Book {
			buys: Vec::new(),
			sells: Vec::new(),
		})
	};
	
	let ord = Order {
		id,
		stock_name,
    	side, 
    	shares,
    	price,
	};
	
	insert_order(book, &ord);

	orders.insert(id, ord);
	
	if attribution {
		//don't actually care
	}

}

fn process_exec(data: &[u8], _stocks: &HashMap<u32, String>, orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, has_price: bool)  {

	let _locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);
	
	let id = as_u64_be8(&data[11..19]);

	let exec_shares = as_u32_be4(&data[19..23]);
	
	let matchnum = as_u64_be8(&data[23..31]);

	let mut book_order = match orders.get_mut(&id) {
		Some(order) => order,
		None => panic!("can't find order {}", id)
	};
	
	let mut side = 'S';
	if book_order.side == 'S' { side = 'B' }; 
	
	if has_price {

		//need to decide what to do with cross trades, ignore this for now
		//probably convert to cancel down and just show a trade later?
		let print_trade = data[31] as char;
		
		if DEBUG { println!("print_trade:{}", print_trade); }
	
		let price = as_u32_be4(&data[32..36]);

		let dprice = price as f32 / 10_000.00;
		if OUTPUT { println!("I {} {} {} {} {}", matchnum, side, exec_shares, book_order.stock_name, dprice); }
		
	} else {

		let dprice = book_order.price as f32 / 10_000.00;
		if OUTPUT { println!("I {} {} {} {} {}", matchnum, side, exec_shares, book_order.stock_name, dprice); }

	}


	book_order.shares = book_order.shares - exec_shares;

	let book = match books.get_mut(&book_order.stock_name) {
		Some(bk) => bk,
		None => panic!("Can't find a book for {}", book_order.stock_name),
	} ;

	if book_order.shares == 0 {
		remove_order(book, book_order);
		orders.remove(&id);
	}

}

fn process_trade(data: &[u8], books: &mut HashMap<String, Book>)  {

	let _locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);

	//these have no values for trade messages
	let _id = as_u64_be8(&data[11..19]);
	let _side = data[19] as char;

	let exec_shares = as_u32_be4(&data[20..24]);

	let stock_name = String::from_utf8_lossy(&data[24..32]).to_string();

	let price = as_u32_be4(&data[32..36]);

	let matchnum = as_u64_be8(&data[36..44]);

	let dprice = price as f32 / 10_000.00;


	let mut side_1 = 'B';
	let mut side_2 = 'S';

	//not sure this handles all situations, might need to actually two pass and stick in before other orrders??
	match books.get(&stock_name){
		Some(book) => {
			// just call the hidden a buy UNLESS the price would collide with sells
			if book.sells.len() > 0 && price >= book.sells[0].price { 
				side_1 = 'S';
				side_2 = 'B';

			}
		},
		None => {},  //could just be totally hidden, then we don't care
	}

	if OUTPUT { println!("H {} {} {} {} {}", matchnum, side_1, exec_shares, stock_name, dprice);	}
	if OUTPUT { println!("I {} {} {} {} {}", matchnum, side_2, exec_shares, stock_name, dprice);	}		
	
}



fn process_cancel(data: &[u8], orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, full: bool) {

	let _locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);
	
	let id = as_u64_be8(&data[11..19]);
	
	let mut book_order = match orders.get_mut(&id) {
		Some(order) => order,
		None => panic!("can't find order {}", id)
	};
	
	let mut cancel_shares = book_order.shares;

	if !full {
		cancel_shares = as_u32_be4(&data[19..23]);
		if OUTPUT { println!("C {} {}", id, cancel_shares); }
	} else { 
		if OUTPUT { println!("X {}", id); }
	}
	
	book_order.shares = book_order.shares - cancel_shares;

	let book = match books.get_mut(&book_order.stock_name) {
		Some(bk) => bk,
		None => panic!("Can't find a book for {}", book_order.stock_name),
	} ;

	if book_order.shares == 0 {
		remove_order(book, book_order);
		orders.remove(&id);
	}

}

fn process_update(data: &[u8], orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>) {

	let _locate = as_u32_be2(&data[1..3]);

	let tracking = as_u32_be2(&data[3..5]);
	if DEBUG { println!("tracking: {}", tracking) };

	let _ts = as_u64_be6(&data[5..11]);
	
	let id = as_u64_be8(&data[11..19]);
	
	let nid = as_u64_be8(&data[19..27]);

	let new_shares = as_u32_be4(&data[27..31]);

	let new_price = as_u32_be4(&data[31..35]);

	let mut book_order = match orders.remove(&id) {
		Some(order) => order,
		None => panic!("can't find order {}", id)
	};

	let book = match books.get_mut(&book_order.stock_name) {
		Some(bk) => bk,
		None => panic!("Can't find a book for {}", book_order.stock_name),
	} ;
	
	remove_order(book, &book_order);
	
	book_order.id = nid;
	book_order.price = new_price;
	book_order.shares = new_shares;

	let dprice = new_price as f32 / 10_000.00;
	
	if OUTPUT { println!("U {} {} {} {}", id, nid, new_shares, dprice);	}

	insert_order(book, &book_order);

	orders.insert(nid, book_order);	

}


fn main() -> io::Result<()> {

    
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
            panic!("Please specifiy an ITCH file");
    }
    
    let filename = &args[1];

    let path = Path::new(filename);
    let display = path.display();
   
    let fil = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut f = BufReader::with_capacity(1024*1024, fil);


    let mut databuf = [0; 1024];

    //probably should initialize this intelligently
    let mut stocks: HashMap<u32, String> = HashMap::with_capacity(10_000);

	let mut orders: HashMap<u64, Order> = HashMap::new();

	let mut counter = 0;

    let mut start = Instant::now();

    let mut books: HashMap<String, Book> =  HashMap::with_capacity(10_000);

    loop {

	    let mut msg_len_b = [0; 2];
	    f.read_exact(&mut msg_len_b)?;  

	    let msg_len = (256 * msg_len_b[0] as u32) + msg_len_b[1] as u32;

	    let msg_len = msg_len as usize;

		f.read_exact(&mut databuf[..msg_len])?;

	    //println!("Len: {}", msg_len);
		let msg_type = databuf[0] as char;
	    
	    if DEBUG {println!("got {} with len {} ", msg_type, msg_len); }

	    match  msg_type {

	    	'S' => match process_system_event(&databuf[..msg_len]) {
				'C' => break,
				_ => continue, //noop	
				},
			
	    	
	    	'R' => process_stock_map(&databuf[..msg_len], &mut stocks),
	    	'A' => process_add_order(&databuf[..msg_len], &mut orders, &mut books, false),
			'F' => process_add_order(&databuf[..msg_len], &mut orders, &mut books, true),

			'E' => process_exec(&databuf[..msg_len], &stocks, &mut orders, &mut books, false),
			'C' => process_exec(&databuf[..msg_len], &stocks, &mut orders, &mut books, true),

			'X' => process_cancel(&databuf[..msg_len],  &mut orders, &mut books, false),
			'D' => process_cancel(&databuf[..msg_len],  &mut orders, &mut books, true),

			'U' => process_update(&databuf[..msg_len],  &mut orders, &mut books),

			'P' => process_trade(&databuf[..msg_len], &mut books),

	    	_ => {
	    		 //don't care for now	
	    		}
	    }
		counter = counter + 1;
		if counter % 100_000 == 0 {
		   let elapsed = start.elapsed();
		   if !OUTPUT { println!("{} {}", counter, elapsed.as_millis() ); }
	    	start = Instant::now();
	    }
	    
	}

    Ok(())
}