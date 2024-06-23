
/*

    This program is intended to be a place where .......

    #Message code table version - 000.01
    00000 - life beat message that broadcast listening port.

*/

//Modules declaration
mod net;
mod block;

use std::time::{Duration, SystemTime};
use std::io; 
use std::thread;
//use net::network::{self, NET_PORT, VERSION};
use net::network;
use std::sync::mpsc::{self, Receiver, Sender};
//use std::sync::mpsc::{self, Receiver, Sender};
use block::{Block, Node};

//const TTT: u8 = network::MAX_PEERS;

//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

//Time constants
const MINUTE: Duration = Duration::from_secs(10);

//const MY_ADDRESS: &str = "xyz6886";


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

        //user_input = user_input.trim().to_string();

        //Send user input to main thread
        if tx.send(user_input).is_err() {
            eprintln!("Failed to send input to main thread.");
            break;
        }
    }  
}

fn handle_thread_msg(message_receiver: &Receiver<String>) -> String{

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            println!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            EMPTY_STRING
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            eprintln!("Input thread has disconnected.");
            EMPTY_STRING
        }
    }
}

fn handle_net_msg(message_receiver: &Receiver<[String; 3]>) -> [String; 3]{

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            println!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            [EMPTY_STRING; 3]
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            eprintln!("Input thread has disconnected.");
            [EMPTY_STRING; 3]
        }
    }
}

fn main() {    
    //Initial greetins
    println!("Welcome to FREDOOM !!!");

    let (input_message, message_receiver) = mpsc::channel();
    let (net_message, net_receiver) = mpsc::channel();

    //Spawn thread for server initialization    
    thread::spawn( move || network::net_init(net_message));

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
        let mut message_buffer: Vec<String> = Vec::new();
        //Control of time triggered features
        match now.elapsed(){

            Ok(n) => {
                println!("Tempo => {:?}", n); //Debug print - to_do change to crate tracer event
                if n >= MINUTE{
                    println!("One minute"); //to_do change to crate tracer event

                    //Propagate self IP address and port
                    //let message = serde_json::to_string(&blocks).expect("Error");
                    let message = serde_json::to_string(&blocks.message).expect("Error");
                    //message.push_str("00001");    //00000 - code for life beat message (check message code table)
                    //message.push_str(VERSION);

                    let msg = message.clone();
                    //Spawn thread to propagate listening port to all network                  
                    thread::spawn(move || network::to_net(msg));

                    now = SystemTime::now();

                }
            },

            Err(e) => println!("Error {}", e),            
        }        

        // Check for new messages from the input thread
        message_buffer.push(handle_thread_msg(&message_receiver));

        // Check for new messages from the network thread
        //message_buffer.push(handle_thread_msg(&net_receiver));

        let mut net_msg: [String; 3] = handle_net_msg(&net_receiver);
        loop {

            println!(" net msg => {}", net_msg[0]);
            if net_msg[0] != EMPTY_STRING {                
    
                if !blocks.message.contains(&net_msg){

                    //Call insert function to format and store in a block section
                    blocks.insert(net_msg.clone());

                }
                

                net_msg = [EMPTY_STRING; 3];
            }
            else {
                break;
            }             
        }

        

        loop{

            let mut user_msg: String = String::new();

            //println!("Debug");
            if let Some(_) =  message_buffer.get(0){

                user_msg = message_buffer.swap_remove(0);

            }
            
            
            if user_msg != EMPTY_STRING {
                //Organize data to fit in the message format [current time, address, message text]
                let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(user_msg.trim())];
                
                //Call insert function to format and store in a block section
                blocks.insert(message.clone());
                //println!("User message => {:?}", message);
            }
            else {
                break;
            }                        

        }
        
        println!(" Blocks => {:?}", blocks.message);
        thread::sleep(Duration::from_millis(3000));       
        
    }

}



/*


        // Check for new messages from the input thread
        let user_msg = handle_thread_msg(&message_receiver);

        // Check for new messages from the network thread
        let net_msg = handle_thread_msg(&net_receiver);

        println!(" net message => {}", net_msg);



*/