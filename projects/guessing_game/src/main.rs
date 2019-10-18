extern crate rand;

use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1, 101); // immutable
    // println!("The secret number is: {}", secret_number);
    loop {
    	println!("Please input your guess.");

	    let mut guess = String::new(); // mutable
	    io::stdin().read_line(&mut guess).expect("Failed to readline");
	    let guess: u32 = match guess.trim().parse() {
	    	Ok(num) => num,
	    	Err(_) => continue,
	    };

	    println!("Your guessed: {}", guess);

	    match guess.cmp(&secret_number) {
	    	Ordering::Less => println!("Too Small!"),
	    	Ordering::Greater => println!("Too Big!"),
	    	Ordering::Equal => {
	    		println!("You Win!");
	    		break; // end of loop
	    	}	
	    }
    }
   
}
