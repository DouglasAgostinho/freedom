
/*
    This program is intended to be a place where all users can present their own models
    or integrate a pool of models for ...
*/

//Modules declaration
mod net;
mod block;
mod crypt;


use std::io; 
//use std::io::{self, Write};
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
fn prog_control() -> (u8, (String, String)) {

    //Clean std out
    //println!("\x1B[2J\x1B[1;1H");

    println!("-----------------------------");
    println!(" !!! Welcome to fredoom !!!");
    println!("-----------------------------");
    println!("Please select an option below.");
    println!("1 - Select model.");
    println!("2 - Check messages List.");
    println!("3 - Check message table");
    println!("4 - Request message.");

    //Variable to receive user input
    let user_input = get_input();

    let u_sel: u8;

    let model_and_message = match &user_input[..] {

        "1" => {

            u_sel = 1;
            
            //println!("\x1B[2J\x1B[1;1H");

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
            //println!("\x1B[2J\x1B[1;1H");
            (EMPTY_STRING, EMPTY_STRING)
        }

        "3" => {
            u_sel = 3;
            //println!("\x1B[2J\x1B[1;1H");
            (EMPTY_STRING, EMPTY_STRING)
        }

        "4" => {
            u_sel = 4;
            //println!("\x1B[2J\x1B[1;1H");
            (EMPTY_STRING, EMPTY_STRING)
        }

        _ => {
            u_sel = 0;
            (EMPTY_STRING, EMPTY_STRING)
        },
    };

    (u_sel, model_and_message)

    
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

        let (action_menu, model_and_message) = prog_control(); 
        
        let mut ser_menu = serde_json::to_string(&model_and_message).expect("error");

        if action_menu == 0 {

            println!("Please select a valid option !");
            
        }
        else {

            ser_menu.push_str(&action_menu.to_string());
            println!(" MSG => {}", ser_menu);

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
        
        Ok(n) => {

            let (stream, ser_msg) = n;

            let msg: (String, String) = serde_json::from_str(&ser_msg)?;

            let (rcv_key, model) = msg;

            println!("received request => {}", model);
            
            for (i, message) in messages.iter().enumerate() {
                println!(" message => {}, model => {}", message[0], model);
                println!("message {:?}, messages {:?}", message, messages);
                if message[0] == model {

                    //stream.write_all(message[1].as_bytes())?;
                    println!("Rceived {:?}", stream.peer_addr());

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
        //Err(io::Error::new(io::ErrorKind::NotFound, "Message not found in messages list !"))
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            //Err(io::Error::new(io::ErrorKind::BrokenPipe, "No input received!"))
            
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


    //Initial greetins (todo main menu)----------------------------------------------------------------------------------
    //println!("Welcome to FREDOOM !!!");
    //println!("\x1B[2J\x1B[1;1H");

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
                println!("User msg => {:?}", user_msg);

                let msg_len = user_msg.len();
                let code = &user_msg[msg_len -1 .. msg_len];  

                println!("Code => {}", code);
                
                match  code {

                    "1" => {

                        let ser_message = user_msg[ .. msg_len -1].to_string();

                        let (selected_model, model_msg): (String, String) = serde_json::from_str(&ser_message).expect("error");

                        model_message.push([selected_model.clone(), model_msg.clone()]);

                        println!("Model Message => {:?}", model_message);


                        //Organize data to fit in the message format [current time, address, message text]
                        let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(selected_model.trim())];

                        //Call insert function to format and store in a block section
                        blocks.insert(message.clone());

                    },

                    "2" => {
                        //let _message_to_model = user_msg[ .. msg_len -1].to_string();

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

                        let model = "llama 3".to_string();
                        //For tests purpose , will be changed to reply when requested a message
                        thread::spawn( move || net::network::request_model_msg("192.168.191.3:6886".to_string(), model));
                        /* 
                        let _ = match net::network::request_model_msg("192.168.191.3:6886".to_string())  {
                            Ok(n) => n,
                            Err(e) =>{
                                error!("Error found while requesting model message => {}", e);
                                EMPTY_STRING
                            }
                        };*/

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
 
        match handle_model_available(&model_receiver, model_message.clone()){
            Ok(i) => {

                if i != 0 {
                    println!("index => {}", i);
                    println!("Messato send => {:?}", model_message[i])
                }
                

            }
            Err(e) => {
                error!("Error while removing message from list => {}", e)
            }
        } 
        
        blocks.message = get_msg_from_blocks(blocks.message, "remove".to_string());
        thread::sleep(Duration::from_millis(1));
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
