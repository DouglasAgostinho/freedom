
/*
    This program is intended to be a place where all users can present their own models
    or integrate a pool of models for ...
*/

//Modules declaration
mod net;
mod block;
mod crypt;


use std::io; 
use std::thread;
use block::{Block, Node};
use net::network::{self, VERSION};
use std::time::{Duration, SystemTime};
use std::sync::mpsc::{self, Receiver, Sender};
use tracing::{span, info, error, Level, instrument};

//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

//Time constants
const MINUTE: Duration = Duration::from_secs(10);

//Logging path constant
const LOG_PATH: &str = "./logs";

#[instrument]
fn get_input() -> String {
    
    let mut user_input = EMPTY_STRING;

    //Get user input
    //println!("Please enter the message");
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => (),
        Err(e) => error!("Error found while getting user input => {}", e),
    }
    user_input.trim().to_string()
}

#[instrument]
fn prog_control() -> (u8, String) {

    //Clean std out
    //println!("\x1B[2J\x1B[1;1H");

    println!("-----------------------------");
    println!(" !!! Welcome to fredoom !!!");
    println!("-----------------------------");
    println!("Please select an option below.");
    println!("1 - Select model.");
    println!("2 - Insert message.");
    println!("3 - Check message table");

    //Variable to receive user input
    let user_input = get_input();

    let u_sel: u8;

    let tx_send = match &user_input[..] {

        "1" => {

            u_sel = 1;
            
            println!("\x1B[2J\x1B[1;1H");

            println!("Please select an option below.");            
            println!("1 - llama 3.");
            println!("2 - Phi 3.");
            println!("3 - Mistral");

            let user_input = get_input();

            let sel = match &user_input[..] {

                "1" => "llama 3",
                "2" => "Phi 3",
                "3" => "Mistral",
                _ => "",                
            };   

            sel.to_string()
        },

        "2" => {
            u_sel = 2;

            println!("Please insert your message");
            let user_input = get_input();
            user_input
        }

        "3" => {
            u_sel = 3;
            println!("\x1B[2J\x1B[1;1H");
            EMPTY_STRING
        }

        _ => {
            u_sel = 0;
            EMPTY_STRING
        },
    };

    (u_sel, tx_send)

    
}


///Receive an item from a Vector of vector(String) if match the NODE Address
fn get_msg_from_blocks(mut block: Vec<[String; 3]>, addr: String) -> Vec<[String; 3]>{

    //Create a vector to receive index of match messages
    let mut to_remove = Vec::new();
    
    //Loop through Block messages to find desired value
    for (num, val) in block.iter().enumerate(){

        if val[2] == addr {
            to_remove.push(num);
        }
    }
    
    //Loop to remove desired messages
    for n in to_remove{
        block.swap_remove(n);
    }

    //Return block without removed messages
    block
}

#[instrument]
fn local_users(tx: Sender<String>){

    loop {

        let (action_menu, mut msg_menu) = prog_control(); 
        
        if action_menu == 0 {

            println!("Please select a valid option !");
            
        }
        else {

            msg_menu.push_str(&action_menu.to_string());
            println!(" MSG => {}", msg_menu);

            match tx.send(msg_menu){
                Ok(t) => t,
                Err(e) => {
                    error!("Failed to send input to main thread => {}", e);
                    break
                }
            }
            
        }
    }  
}

#[instrument] //Tracing auto span generation
fn handle_thread_msg(message_receiver: &Receiver<String>) -> String{

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            EMPTY_STRING
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            EMPTY_STRING
        }
    }
}

#[instrument] //Tracing auto span generation
fn handle_net_msg(message_receiver: &Receiver<[String; 3]>) -> [String; 3]{

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            [EMPTY_STRING; 3]
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            [EMPTY_STRING; 3]
        }
    }
}

fn main() {

    //Instatiate the subscriber & file appender
    let file_appender = tracing_appender::rolling::hourly(LOG_PATH, "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();

    //Entering Main Loggin Level
    let span: span::Span = span!(Level::INFO,"Main");
    let _enter: span::Entered = span.enter();


    //Initial greetins (todo main menu)----------------------------------------------------------------------------------
    //println!("Welcome to FREDOOM !!!");
    println!("\x1B[2J\x1B[1;1H");

    //Initiate Thread message channel Tx / Rx 
    let (input_message, message_receiver) = mpsc::channel();
    let (net_message, net_receiver) = mpsc::channel();

    let sspan = span.clone();
    //Spawn thread for server initialization
    thread::spawn( move || sspan.in_scope(move || network::net_init(net_message)));

    //Instance of Block struct
    let mut blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Instance of Node struct
    let mut my_node: Node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();

    //Initiate time measurement - for time triggered features
    let mut now = SystemTime::now();

    //Spawn thread for handle local user interaction
    thread::spawn(move || {local_users(input_message)});

    loop{

        //Buffer to store received messages
        let mut message_buffer: Vec<String> = Vec::new();

        //Control of time triggered features
        match now.elapsed(){

            Ok(n) => {
                info!("Tempo => {:?}", n);
                if n >= MINUTE{
                    info!("One minute");

                    //Propagate message block
                    let mut message = match serde_json::to_string(&blocks.message){
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("Error while serializing Block propagation message {}", e);
                            EMPTY_STRING
                        },
                    };
                    message.push_str("00001");    //00001 - code for block propagation (check message code table)
                    message.push_str(VERSION);

                    let msg = message.clone();
                    //Spawn thread to propagate listening port to all network
                    thread::spawn(move || network::to_net(msg));

                    now = SystemTime::now();
                }
            },
            Err(e) => error!("Error {}", e),
        }

        // Check for new messages from the input thread
        message_buffer.push(handle_thread_msg(&message_receiver));

        // Check for new messages from the network thread
        let mut net_msg: [String; 3] = handle_net_msg(&net_receiver);

        loop {

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

            if let Some(_) =  message_buffer.get(0){

                user_msg = message_buffer.swap_remove(0);
            }

            if user_msg != EMPTY_STRING {

                let msg_len = user_msg.len();
                let code = &user_msg[msg_len -1 .. msg_len];  
                
                match  code {

                    "1" => {

                        let selected_model = user_msg[ .. msg_len -1].to_string();

                        //Organize data to fit in the message format [current time, address, message text]
                        let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(selected_model.trim())];

                        //Call insert function to format and store in a block section
                        blocks.insert(message.clone());

                    },

                    "2" => {
                        //let _message_to_model = user_msg[ .. msg_len -1].to_string();


                        //For tests purpose , will be changed to reply when requested a message

                        let _ = match net::network::request_model_msg("192.168.191.2:6886".to_string())  {
                            Ok(n) => n,
                            Err(e) =>{
                                error!("Error found while requesting model message => {}", e);
                                EMPTY_STRING
                            }
                        };
                    },

                    "3" => {

                        println!("-----------------------------");
                        println!("  !!!  Updated blocks  !!!");
                        println!("-----------------------------");
                        
                        println!(" Blocks => {:?}", blocks.message);
                    }

                    _ => (),                    
                }
                
            }
            else {
                break;
            }
        }

        //Clean std out
        //println!("\x1B[2J\x1B[1;1H");

        
        blocks.message = get_msg_from_blocks(blocks.message, "remove".to_string());
        //thread::sleep(Duration::from_millis(3000));
/*
        let _ = match net::network::request_model_msg("192.168.191.2:6886".to_string())  {
            Ok(n) => n,
            Err(e) =>{
                error!("Error found while requesting model message => {}", e);
                EMPTY_STRING
            }
        };
 */
    }

}
