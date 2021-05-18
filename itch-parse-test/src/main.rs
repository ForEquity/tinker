
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env;

use std::io::BufReader;
use std::io::BufRead;


use std::time::Instant;

use std::collections::hash_map::Entry;


use std::collections::HashMap;

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

fn as_u64_be6(array: &[u8; 6]) -> u64 {
    ((array[0] as u64) << 40) +
    ((array[1] as u64) << 32) +
    ((array[2] as u64) << 24) +
    ((array[3] as u64) << 16) +
    ((array[4] as u64) <<  8) +
    ((array[5] as u64) <<  0)
}

fn as_u64_be8(array: &[u8; 8]) -> u64 {
	((array[0] as u64) << 56) +
    ((array[1] as u64) << 48) +
    ((array[2] as u64) << 40) +
    ((array[3] as u64) << 32) +
    ((array[4] as u64) << 24) +
    ((array[5] as u64) << 16) +
    ((array[6] as u64) <<  8) +
    ((array[7] as u64) <<  0)
}

fn as_u32_be4(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

fn as_u32_be2(array: &[u8; 2]) -> u32 {
    ((array[0] as u32) <<  8) +
    ((array[1] as u32) <<  0)
}


fn extract_timestamp<R: BufRead>(f: &mut R) -> u32 {

	//just skiping it for now
	let mut timestamp_buffer = [0; 6];
	match f.read_exact(&mut timestamp_buffer) {
		Err(_why) => return 0,
		Ok(_read) => return 0
	}

}

fn process_system_event<R: BufRead>(f: &mut R) -> io::Result<char> {

	//just want the event code
	let mut skip = [0; 10];
	f.read_exact(&mut skip)?;
	
	let mut event_code = [0; 1];
	f.read_exact(&mut event_code)?;  

	let ec = event_code[0] as char;

	if OUTPUT { println!("Event Code:{}", ec);}
	Ok(ec)
}

fn process_stock_map<R: BufRead>(f: &mut R, stocks: &mut HashMap<u32, String>) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let locate = as_u32_be2(&locate);

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);
	
	let mut stock_buff = [0; 8];
	f.read_exact(&mut stock_buff)?;

	let stock_name = String::from_utf8_lossy(&stock_buff).to_string();
	
	if OUTPUT { println!("Adding {} at {}", stock_name, locate); }

	stocks.insert(locate, stock_name);
	
	//don't care about the rest
	let mut skip = [0; 20];
	f.read_exact(&mut skip)?;

	Ok(())
}

fn process_add_order<R: BufRead>(f: &mut R, orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, attribution: bool) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let _locate = as_u32_be2(&locate) as usize;

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);

	//will need this eventually for some better magic
	let mut orn = [0; 8];
	f.read_exact(&mut orn)?;  
	let orn = as_u64_be8(&orn);
	
	let mut side = [0; 1];
	f.read_exact(&mut side)?;  
	let side = side[0] as char;

	let mut shares = [0; 4];
	f.read_exact(&mut shares)?;  
	let shares = as_u32_be4(&shares) as u32;

	let mut stock_buff = [0; 8];
	f.read_exact(&mut stock_buff)?;
	let stock_name = String::from_utf8_lossy(&stock_buff).to_string();

	let mut price = [0; 4];
	f.read_exact(&mut price)?;  
	let price = as_u32_be4(&price) as u32;
	let dprice = price as f32 / 10_000.00;

	if OUTPUT { println!("A {} {} {} {} {}", orn, side, shares, stock_name, dprice);} 


//	let &mut book = Book {
//			buys: Vec::new(),
//			sells: Vec::new(),
//	};

//	let mut newbook = false;

//	if books.contains_key(&stock_name) {
//		book = match books.get_mut(&stock_name) {
//			Some(bk) => bk,
//			None => {panic!("Said it was there but couldn't get {}", stock_name)},
//		}
//	}

	let stock_key = stock_name.clone();

	let book: &mut Book = match books.entry(stock_key) {
   		Entry::Occupied(o) => o.into_mut(),
   		Entry::Vacant(v) => v.insert(Book {
			buys: Vec::new(),
			sells: Vec::new(),
		})
	};
	
	let ord = Order {
		id: orn,
		stock_name,
    	side, 
    	shares,
    	price,
	};
	
	insert_order(book, &ord);

	orders.insert(orn, ord);
	
	if attribution {
		let mut skip = [0; 4];
		f.read_exact(&mut skip)?;  
	}

	Ok(())
}

fn process_exec<R: BufRead>(f: &mut R, _stocks: &HashMap<u32, String>, orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, has_price: bool) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let _locate = as_u32_be2(&locate);

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);

	let mut orn = [0; 8];
	f.read_exact(&mut orn)?;  
	let orn = as_u64_be8(&orn);

	let mut exec_shares = [0; 4];
	f.read_exact(&mut exec_shares)?;  
	let exec_shares = as_u32_be4(&exec_shares);

	let mut matchnum = [0; 8];
	f.read_exact(&mut matchnum)?;  
	let matchnum = as_u64_be8(&matchnum);


	let mut book_order = match orders.get_mut(&orn) {
		Some(order) => order,
		None => panic!("can't find order {}", orn)
	};

	//hack for testing
	//let o = Order {stock_name:  String::new(), price: 0, shares: 0, side: 'Z'};

	//ok this hit a lit order, pull it out get the price and side

	let mut side = 'S';
	if book_order.side == 'S' { side = 'B' }; 

	
	if has_price {

		//need to decide what to do with cross trades, ignore this for now
		//probably convert to cancel down and just show a trade later?
		let mut print_trade = [0; 1];
		f.read_exact(&mut print_trade)?;  

		if DEBUG { println!("print_trade:{}", print_trade[0] as char); }
	
		let mut price = [0; 4];
		f.read_exact(&mut price)?;  
		let price = as_u32_be4(&price);

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
		orders.remove(&orn);
	}



	Ok(())
}

fn process_trade<R: BufRead>(f: &mut R, books: &mut HashMap<String, Book>) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let _locate = as_u32_be2(&locate);

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);

	let mut orn = [0; 8];
	f.read_exact(&mut orn)?;  
	let orn = as_u64_be8(&orn);

	if DEBUG {println!("trade orn:{}", orn); }

	let mut side = [0; 1];
	f.read_exact(&mut side)?;  
	let side = side[0] as char;

	if DEBUG { println!("trade side:{}", side); }

	let mut exec_shares = [0; 4];
	f.read_exact(&mut exec_shares)?;  
	let exec_shares = as_u32_be4(&exec_shares);

	let mut stock_buff = [0; 8];
	f.read_exact(&mut stock_buff)?;
	let stock_name = String::from_utf8_lossy(&stock_buff).to_string();

	let mut price = [0; 4];
	f.read_exact(&mut price)?;  
	let price = as_u32_be4(&price);

	let mut matchnum = [0; 8];
	f.read_exact(&mut matchnum)?;  
	let matchnum = as_u64_be8(&matchnum);

	let dprice = price as f32 / 10_000.00;


	let mut side_1 = 'B';
	let mut side_2 = 'S';


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

	Ok(())
}



fn process_cancel<R: BufRead>(f: &mut R, orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>, full: bool) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let _locate = as_u32_be2(&locate);

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);

	let mut orn = [0; 8];
	f.read_exact(&mut orn)?;  
	let orn = as_u64_be8(&orn);

	let mut book_order = match orders.get_mut(&orn) {
		Some(order) => order,
		None => panic!("can't find order {}", orn)
	};
	
	let mut cancel_shares = book_order.shares;

	if !full {
		let mut cancel_sharesb = [0; 4];
		f.read_exact(&mut cancel_sharesb)?;  
		cancel_shares = as_u32_be4(&cancel_sharesb);
		if OUTPUT { println!("C {} {}", orn, cancel_shares); }
	} else { 
		if OUTPUT { println!("X {}", orn); }
	}
	
	book_order.shares = book_order.shares - cancel_shares;

	let book = match books.get_mut(&book_order.stock_name) {
		Some(bk) => bk,
		None => panic!("Can't find a book for {}", book_order.stock_name),
	} ;

	if book_order.shares == 0 {
		remove_order(book, book_order);
		orders.remove(&orn);
	}


	Ok(())
}

fn process_update<R: BufRead>(f: &mut R, orders: &mut HashMap<u64, Order>, books: &mut HashMap<String, Book>) -> io::Result<()> {

	let mut locate = [0; 2];
	f.read_exact(&mut locate)?;  

	let _locate = as_u32_be2(&locate);

	//don't think i need to care
	let mut tracking = [0; 2];
	f.read_exact(&mut tracking)?;  

	if DEBUG { println!("tracking: {}", as_u32_be2(&tracking)) };

	let _ts = extract_timestamp(f);

	let mut orn = [0; 8];
	f.read_exact(&mut orn)?;  
	let orn = as_u64_be8(&orn);

	let mut norn = [0; 8];
	f.read_exact(&mut norn)?;  
	let norn = as_u64_be8(&norn);


	let mut book_order = match orders.remove(&orn) {
		Some(order) => order,
		None => panic!("can't find order {}", orn)
	};

	let book = match books.get_mut(&book_order.stock_name) {
		Some(bk) => bk,
		None => panic!("Can't find a book for {}", book_order.stock_name),
	} ;

	
	remove_order(book, &book_order);
	
	let mut new_shares = [0; 4];
	f.read_exact(&mut new_shares)?;  
	let new_shares = as_u32_be4(&new_shares);
	
	let mut new_price = [0; 4];
	f.read_exact(&mut new_price)?;  
	let new_price = as_u32_be4(&new_price);

	book_order.price = new_price;
	book_order.shares = new_shares;

	let dprice = new_price as f32 / 10_000.00;
	
	if OUTPUT { println!("U {} {} {} {}", orn, norn, new_shares, dprice);	}

	insert_order(book, &book_order);

	orders.insert(norn, book_order);	

	
	Ok(())
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


    let mut skip = [0; 1024];

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

	    //println!("Len: {}", msg_len);

	    let mut msg_type = [0; 1];

	    f.read_exact(&mut msg_type)?;  

	    if DEBUG {println!("got {} with len {} ", msg_type[0] as char, msg_len); }

	    match msg_type[0] as char {

	    	'S' => match process_system_event(&mut f){
	    		Ok('C') => break,
	    		Err(why) => panic!("couldn't read: {}", why),
	    		_ => {}

	    	},
	    	
	    	'R' => process_stock_map(&mut f, &mut stocks)?,
	    	'A' => process_add_order(&mut f, &mut orders, &mut books, false)?,
			'F' => process_add_order(&mut f, &mut orders, &mut books, true)?,

			'E' => process_exec(&mut f, &stocks, &mut orders, &mut books, false)?,
			'C' => process_exec(&mut f, &stocks, &mut orders, &mut books, true)?,

			'X' => process_cancel(&mut f,  &mut orders, &mut books, false)?,
			'D' => process_cancel(&mut f,  &mut orders, &mut books, true)?,

			'U' => process_update(&mut f,  &mut orders, &mut books)?,

			'P' => process_trade(&mut f, &mut books)?,

	    	_ => {
	    		f.read_exact(&mut skip[..msg_len-1])?;  //don't care for now	
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