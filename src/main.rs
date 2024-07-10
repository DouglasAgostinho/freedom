
/*

    //Clean std out
    //println!("\x1B[2J\x1B[1;1H");

*/
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
use std::net::TcpStream;
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

//Own Static IP and PORT
const MY_ADDRESS: &str = "192.168.191.2:6886";

#[instrument]
fn get_input() -> String {
    
    let mut user_input = EMPTY_STRING;

    match io::stdin().read_line(&mut user_input) {
        Ok(_) => (),
        Err(e) => error!("Error found while getting user input => {}", e),
    }
    user_input.trim().to_string()
}

#[instrument]
fn prog_control() -> (u8, (String, String)) {

    println!("-----------------------------");
    println!(" !!! Welcome to fredoom !!!");
    println!("-----------------------------");
    println!("Please select an option below.");
    println!("1 - Select model.");
    println!("2 - Check messages List.");
    println!("3 - Check message table");
    println!("4 - Request message.");
    println!("----- Local model (to do) -----.");

    //Variable to receive user input
    let user_input = get_input();

    let u_sel: u8;

    let model_and_message = match &user_input[..] {

        "1" => {

            u_sel = 1;
            
            println!("Please select an option below.");            
            println!("1 - llama 3.");
            println!("2 - Phi 3.");
            println!("3 - Mistral");

            let user_input = get_input();

            let model = match &user_input[..] {

                "1" => "llama 3",
                "2" => "Phi 3",
                "3" => "Mistral",
                _ => "",                
            };   

            println!("Now, please insert your message");
            let user_input = get_input();
            

            (model.to_string(), user_input)
        },

        "2" => {
            u_sel = 2;
            (EMPTY_STRING, EMPTY_STRING)
        }

        "3" => {
            u_sel = 3;
            (EMPTY_STRING, EMPTY_STRING)
        }

        "4" => {
            u_sel = 4;
            (EMPTY_STRING, EMPTY_STRING)
        }

        _ => {
            u_sel = 0;
            (EMPTY_STRING, EMPTY_STRING)
        },
    };

    (u_sel, model_and_message)
}

/*
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
*/

#[instrument]
fn local_users(tx: Sender<String>){

    loop {

        let (action_menu, model_and_message) = prog_control(); 
        
        let mut ser_menu = serde_json::to_string(&model_and_message).expect("error");

        if action_menu == 0 {
            println!("Please select a valid option !");
        }
        else {
            ser_menu.push_str(&action_menu.to_string());

            match tx.send(ser_menu){
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

#[instrument] //Tracing auto span generation
fn handle_model_available(model_receiver: &Receiver<(TcpStream, String)>, messages: Vec<[String; 2]>) -> io::Result<usize>{
       
    match model_receiver.try_recv() {
        
        Ok((stream, ser_msg)) => {

            let msg: (String, String) = serde_json::from_str(&ser_msg)?;

            let (rcv_key, model) = msg;
            
            for (i, message) in messages.iter().enumerate() {
                
                if message[0] == model {

                    let _ = match net::network::send_model_msg(rcv_key, message[1].to_string(), stream)  {
                        Ok(n) => n,
                        Err(e) =>{
                            error!("Error found while requesting model message => {}", e);
                            EMPTY_STRING
                        }
                    };
                    return Ok(i);
                }
            }   
            Ok(0)
        },
        Err(mpsc::TryRecvError::Empty) => {
            Ok(0)
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Input thread has disconnected!"))
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

    //Initiate Thread message channel Tx / Rx 
    let (input_message, message_receiver) = mpsc::channel();
    let (net_message, net_receiver) = mpsc::channel();
    let (model_sender, model_receiver) = mpsc::channel();
    

    let sspan = span.clone();
    //Spawn thread for server initialization
    thread::spawn( move || sspan.in_scope(move || network::net_init(net_message, model_sender)));

    let mut model_message: Vec<[String; 2]> = Vec::from([[EMPTY_STRING; 2]]); 

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
                //info!("Tempo => {:?}", n);
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

                        let ser_message = user_msg[ .. msg_len -1].to_string();

                        let (selected_model, model_msg): (String, String) = serde_json::from_str(&ser_message).expect("error");

                        model_message.push([selected_model.clone(), model_msg.clone()]);

                        //Organize data to fit in the message format [current time, address, message text]
                        //let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(selected_model.trim())];
                        let message: [String; 3] = [my_node.get_time_ns(), MY_ADDRESS.to_string(), String::from(selected_model.trim())];

                        //Call insert function to format and store in a block section
                        blocks.insert(message.clone());

                    },

                    "2" => {
                        println!("-----------------------------");
                        println!("  !!!   Messages List   !!!");
                        println!("-----------------------------");
                        println!(" Messages => {:?}", model_message);
                    },

                    "3" => {
                        println!("-----------------------------");
                        println!("  !!!  Updated blocks  !!!");
                        println!("-----------------------------");
                        println!(" Blocks => {:?}", blocks.message);
                    }

                    "4" => {

                        //let remote_server_ip = "192.168.191.2:6886".to_string();
                        let owned_model = "Phi 3".to_string();

                        for msg in blocks.message.iter(){

                            if msg[2] == owned_model{

                                let remote_server_ip = msg[1].clone();
                                let model = msg[2].clone();

                                match TcpStream::connect(&remote_server_ip){
                                    Ok(_) => {
                                        thread::spawn(move || net::network::request_model_msg(remote_server_ip, model));
                                    },
                                    Err(e) => {
                                        println!("Server not available, try later!");
                                        error!("Error while checking server connectivity => {}", e)
                                    },
                                }
                            }
                        }
                    }
                    _ => (),                    
                }
                
            }
            else {
                break;
            }
        }
 
        match handle_model_available(&model_receiver, model_message.clone()){
            Ok(_) => {
                info!("Message processed")

            }
            Err(e) => {
                error!("Error while removing message from list => {}", e)
            }
        } 
        
        //blocks.message = get_msg_from_blocks(blocks.message, "remove".to_string());
        thread::sleep(Duration::from_millis(1));
    }

}
