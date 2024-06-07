
mod net;

use std::io;

use net::network;

fn main() {
    println!("Welcome to FREDOOM !!!");

    //network::net_init();

    let mut message = String::new();

    let address = String::from("192.168.191.1:8687");

    match io::stdin().read_line(&mut message) {
        Ok(_) => (),
        Err(e) => println!("Error found {}", e),
        
    }

    println!("{}", message);

    match network::client(&message, &address){
        Ok(_) => (),
        Err(e) => println!("Error found {}", e),
    }

}
