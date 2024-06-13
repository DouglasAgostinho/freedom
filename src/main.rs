
/*

    This program is intended to be a place where .......

    #Message code table
    000 - life beat message that broadcast listening port.

*/

//Modules declaration
mod net;
mod block;

use std::time::{Duration, SystemTime};
use std::io; 
use std::thread;
use net::network;
use std::sync::mpsc::{self, Sender};
//use std::sync::mpsc::{self, Receiver, Sender};
use block::{Block, Node};


//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

//Time constants
const MINUTE: Duration = Duration::from_secs(60);


//Constant Address & PORT
const NET_PORT: &str = "6886";

fn local_users(tx: Sender<String>){
    
    loop {                
        //Variable to receive user input
        let mut user_input = EMPTY_STRING;

        //Get user input
        println!("Please enter the message");
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => (),
            Err(e) => println!("Error found {}", e),
        }        

        //Send user input to main thread
        if tx.send(user_input).is_err() {
            eprintln!("Failed to send input to main thread.");
            break;
        }
    }  
}

fn main() {
    //Initial greetins
    println!("Welcome to FREDOOM !!!");

    let (input_message, message_receiver) = mpsc::channel();

    //Spawn thread for server initialization    
    thread::spawn( || network::net_init(NET_PORT));

    //Instance of Block struct
    let mut blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Instance of Node struct
    let mut my_node: Node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();

    //Initiate time measurement - some features will be time triggered     
    let mut now = SystemTime::now();

    //Spawn thread for handle local user interaction
    thread::spawn(move || {local_users(input_message)});

    loop{

        //Control of time triggered features
        match now.elapsed(){

            Ok(n) => {
                println!("Tempo => {:?}", n); //Debug print - to_do change to crate tracer event
                if n >= MINUTE{
                    println!("One minute"); //to_do change to crate tracer event

                    //Propagate self IP address and port
                    //let message = serde_json::to_string(&blocks).expect("Error");

                    //Composing message
                    let mut message: String = String::from(NET_PORT);
                    message.push_str("000");    //000 - code for life beat message (check message code table)

                    //Spawn thread to propagate listening port to all network                  
                    thread::spawn(move || network::to_net(&message));

                    now = SystemTime::now();

                }
            },

            Err(e) => println!("Error {}", e),
            
        }        

        // Check for new messages from the input thread
        let user_input = match message_receiver.try_recv() {
            Ok(input) => {
                //Return input received
                println!("Received input: {:?}", input);
                input
            },
            Err(mpsc::TryRecvError::Empty) => {
                // No input received, return Empty String 
                EMPTY_STRING
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Input thread has disconnected.");
                break;
            }
        };
               

        if user_input != EMPTY_STRING {
            //Organize data to fit in the message format [current time, address, message text]
            let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(user_input.trim())];

            //Call insert function to format and store in a block section
            blocks.insert(message.clone());
        }                

        //println!("{:?}", blocks.message );

        

        thread::sleep(Duration::from_millis(3000));

        

        
    }

}


