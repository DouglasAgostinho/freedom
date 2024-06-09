
mod net;
mod block;

use std::io;

//use net::network;
use block::Letter;


const EMPTY_STRING: String = String::new();

fn main() {
    println!("Welcome to FREDOOM !!!");

    //network::net_init();

    let mut message: [String; 3] = [EMPTY_STRING; 3];
    let index: [&str; 3] = ["Time", "Addr", "message"];

    let mut item = String::new();

    let mut my_block = Letter{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //let address = String::from("192.168.191.1:8687");

    loop{

        for i in 0..3{

            println!("Please enter {}", index[i]);
    
            match io::stdin().read_line(&mut item) {
                Ok(_) => (),
                Err(e) => println!("Error found {}", e),
                
            }        
    
            message[i] = String::from(item.trim());
    
            item = String::from("");
        }
    
        println!("Print Vector {:?}", message);

        println!("Print Vector {:?}", my_block.message);

        //my_block.message.push(message.clone());

        my_block.add(message.clone());

        println!("Print Vector {:?}", my_block.message);

    }

    
    

    /*println!("{}", message);

    match network::client(&message, &address){
        Ok(_) => (),
        Err(e) => println!("Error found {}", e),
    }
    */
}
