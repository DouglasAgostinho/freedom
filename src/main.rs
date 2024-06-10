

mod net;
mod block;


use std::io; 

//use net::network;
use block::{Block, Node};
//use serde::{Deserialize, Serialize};


const EMPTY_STRING: String = String::new();


// This is the main function
fn main() {

    println!("Welcome to FREDOOM !!!");

    //network::net_init();

    //let mut message: [String; 3] = [EMPTY_STRING; 3];
    //let index: [&str; 3] = ["Time", "Addr", "message"];

    let mut item = String::new();

    let mut my_block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    let mut my_node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();

    //let address = String::from("192.168.191.1:8687");

    loop{        

        println!("Please enter the message");

        match io::stdin().read_line(&mut item) {
            Ok(_) => (),
            Err(e) => println!("Error found {}", e),
            
        }        

        let message = [my_node.get_time_ns(), my_node.address.clone(), String::from(item.trim())];

        item = String::from("");        
    
        println!("Print Vector {:?}", message);

        println!("Print Vector {:?}", my_block.message);

        //my_block.message.push(message.clone());

        my_block.insert(message.clone());

        println!("Print Vector {:?}", my_block.message);

        println!("Pre Serde {:?}", my_block);
        let v = serde_json::to_value(&my_block).unwrap();
        
        let document: Block = serde_json::from_value(v).unwrap();
        println!("Pos Serde {:?}", document);
    }

}


/*
mod net;
mod block;

use std::io;
//use net::network;
use block::Block;

const EMPTY_STRING: String = String::new();


fn main() {
    println!("Welcome to FREDOOM !!!");

    //network::net_init();

    let mut message: [String; 3] = [EMPTY_STRING; 3];
    let index: [&str; 3] = ["Time", "Addr", "message"];

    let mut item = String::new();

    let mut my_block = Block{
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

*/    
    

    /*println!("{}", message);

    match network::client(&message, &address){
        Ok(_) => (),
        Err(e) => println!("Error found {}", e),
    }
    */

