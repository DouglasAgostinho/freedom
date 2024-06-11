//Modules declaration
mod net;
mod block;

use std::io; 
use std::thread;
use net::network;
use block::{Block, Node};

//Constant to use in String based variables
const EMPTY_STRING: String = String::new();


fn main() {
    //Initial greetins
    println!("Welcome to FREDOOM !!!");

    //Spawn thread for server initialization    
    thread::spawn( || network::net_init());

    //Instance of Block struct
    let mut blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Instance of Node struct
    let mut my_node: Node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();

    loop{

        //Variable to receive user input
        let mut user_input = EMPTY_STRING;

        //Get user input
        println!("Please enter the message");
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => (),
            Err(e) => println!("Error found {}", e),
        }        

        //Organize data to fit in the message format [current time, address, message text]
        let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(user_input.trim())];       

        //Call insert function to format and store in a block section
        blocks.insert(message.clone());

        println!("{:?}", blocks.message );
    }

}


