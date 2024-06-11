//Modules declaration
mod net;
mod block;

use core::time;
use std::io; 
use std::thread;
use net::network;
use std::sync::mpsc::{self, Sender};
//use std::sync::mpsc::{self, Receiver, Sender};
use block::{Block, Node};


//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

fn handle_input(node: &Node, tx: Sender<[String; 3]>){

    loop {                
        //Variable to receive user input
        let mut user_input = EMPTY_STRING;

        //Get user input
        println!("Please enter the message");
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => (),
            Err(e) => println!("Error found {}", e),
        }        

        //Organize data to fit in the message format [current time, address, message text]
        let message: [String; 3] = [node.get_time_ns(), node.address.clone(), String::from(user_input.trim())];

        if tx.send(message).is_err() {
            eprintln!("Failed to send input to main thread.");
            break;
        }
    }
    
}

fn main() {
    //Initial greetins
    println!("Welcome to FREDOOM !!!");

    let (send_to_main, main_receiver) = mpsc::channel();

    //Spawn thread for server initialization    
    thread::spawn( || network::net_init());

    //Instance of Block struct
    let mut blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Instance of Node struct
    let mut my_node: Node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();


    thread::spawn(move || {handle_input(&my_node, send_to_main)});

    loop{

        // Check for new messages from the input thread
        let message = match main_receiver.try_recv() {
            Ok(input) => {
                println!("Received input: {:?}", input);
                input
            },
            Err(mpsc::TryRecvError::Empty) => {
                // No input received, continue with other work
                continue;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Input thread has disconnected.");
                break;
            }
        };
               

        //Call insert function to format and store in a block section
        blocks.insert(message.clone());

        println!("{:?}", blocks.message );

        thread::sleep(time::Duration::from_millis(2000));
    }

}


