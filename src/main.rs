
mod net;

use std::io;

use net::network;

fn main() {
    println!("Welcome to FREDOOM !!!");

    //network::net_init();

    let mut message = String::new();

    match io::stdin().read_line(&mut message) {
        Ok(_) => (),
        Err(e) => println!("Error found {}", e),
        
    }

    println!("{}", message);

}
