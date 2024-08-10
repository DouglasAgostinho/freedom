
/*
    This program is intended to be a place where all users can present their own models
    or integrate a pool of models for research purposes

    It will consist of:
    - Network module that will manage the message communication between peers;
    - Crypt module that will handle the message encryption between peers;
    - Block where most structs will be built;
    - Web server to handle User interaction (external / internal);
    - Main loop:
        - handle the messages from network / web / local CLI
        - control communication with available models & peers
*/

//Modules declaration
mod net;
mod block;
mod crypt;


use std::io; 
use std::thread;
use block::NetWorkMessage;
use serde::Deserialize;
use std::net::TcpStream;
use block::{Block, Node, Peers};
use net::network::{self, VERSION};
use std::time::{Duration, SystemTime};
use std::sync::mpsc::{self, Receiver, Sender};
use tracing::{span, info, error, Level, instrument};

//use tokio_console::config_reference::ConsoleLayer;

use axum::{
    extract::State, //to share variables between routes
    extract::Form,  //to get values from post method on web page
    response::{Html, IntoResponse}, 
    routing::{get, get_service}, Router 
};

use tower_http::services::ServeDir; //Aux functions for AXUM server
use tokio::sync::Mutex;
use std::sync::Arc;


const GREETINGS: &str = 
"
    ##################################
    #                                #
    # -----------------------------  #
    #  !!! Welcome to fredoom !!!    #
    # -----------------------------  #
    #                                #
    ##################################
";

const MAIN_MENU: &str = 
"\n\n
    Please select an option below.
    1 - Select model.
    2 - Check own messages List.
    3 - Check message table.
    4 - Request message.
        -- Local model (to do) --
";

//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

//Time constants
const MINUTE: Duration = Duration::from_secs(10);

//Logging path constant
const LOG_PATH: &str = "./logs";

//Own Static IP and PORT
const MY_ADDRESS: &str = "192.168.191.2:6886";

//Struc of Data from Web Server
#[derive(Deserialize, Debug)] 
struct FormData {
    message: String,
    model: String,
    hidden_content: String,
}

//Struct of variables that will be shared with the web routes (pages)
#[derive(Clone)]
struct AppState{
    model_message: Arc<Mutex<String>>,
    web_info: Arc<Mutex<Vec<FormData>>>,
}

//Get user input from std in and return it
#[instrument]
fn get_input() -> String {
    
    let mut user_input = EMPTY_STRING;

    match io::stdin().read_line(&mut user_input) {
        Ok(_) => (),
        Err(e) => error!("Error found while getting user input => {}", e),
    }
    user_input.trim().to_string()
}


// Present a CLI based menu and get user selection
//
// If option (1): 
//  - get the user input as message to be sent to model
//  - get selected model and return a tuple (model, message) 
//
#[instrument]
fn prog_control() -> (u8, (String, String)) {

    //println!("\x1B[2J\x1B[1;1H");
    println!("{}", GREETINGS);
    println!("{}", MAIN_MENU);

    //Variable to receive user input
    let user_input = get_input();
    println!("User Input is => {}", user_input);

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


#[instrument]
fn local_users(tx: Sender<(u8, (String, String))>){
    //fn local_users(tx: Sender<String>){

    loop {

        //let (action_menu, model_and_message) = prog_control(); 
        let ser_menu = prog_control(); 

        //println!("User menu is => {}", ser_menu.0);

        //let mut ser_menu = serde_json::to_string(&model_and_message).expect("error");

        if ser_menu.0 == 0 {
            //if action_menu == 0 {
            println!("Please select a valid option !");
        }
        else {
            //ser_menu.push_str(&action_menu.to_string());
            //println!("Ser menu => {:?}", ser_menu);

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
fn handle_thread_msg(message_receiver: &Receiver<(u8, (String, String))>) -> (u8, (String, String)){
    //fn handle_thread_msg(message_receiver: &Receiver<String>) -> String{

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            (0, (EMPTY_STRING, EMPTY_STRING))
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            (0, (EMPTY_STRING, EMPTY_STRING))
        }
    }
}

#[instrument] //Tracing auto span generation
fn handle_net_msg(message_receiver: &Receiver<NetWorkMessage>) -> (Vec<[String; 3]>, Vec<Peers>){
    //fn handle_net_msg(message_receiver: &Receiver<[String; 3]>) -> [String; 3]{

    let msg = match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            //[EMPTY_STRING; 3]
            NetWorkMessage::new()
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            //[EMPTY_STRING; 3]
            NetWorkMessage::new()
        }
    };

    let net_message: Vec<[String; 3]> = if msg.message != EMPTY_STRING{
        match serde_json::from_str(&msg.message){

            Ok(msg) => msg,
            Err(e) => {
                error!("Error while deserializing Net Message => {}", e);
                Vec::from([[EMPTY_STRING; 3]])
            }
        }
    }
    else{
        Vec::from([[EMPTY_STRING; 3]])
    };

    (net_message, msg.peers)

}

fn le_model (model_rx: &Receiver<String>) -> String{

        let mmsg = match model_rx.try_recv() {
            Ok(msg) => {
                //Return input received
                info!("Received input: {:?}", msg);
                //println!("Le model => {:?}", msg);
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
        };
        mmsg
}

#[instrument] //Tracing auto span generation
fn handle_model_available(
    model_receiver: &Receiver<(TcpStream, String)>, 
    mut messages: Vec<[String; 2]>,
    tx_model: Sender<String>
    ) -> io::Result<usize>{

    match model_receiver.try_recv() {

        Ok((stream, ser_msg)) => {

            let msg: (String, String) = serde_json::from_str(&ser_msg)?;

            let (rcv_key, model) = msg;

            for (i, message) in messages.iter().enumerate() {

                if message[0] == model {
                    let msg = messages.swap_remove(i);
                    thread::spawn( move ||
                    match net::network::send_model_msg(rcv_key, msg[1].to_string(), stream, tx_model) {
                        Ok(n) => {n},
                        Err(e) =>{
                            error!("Error found while requesting model message => {}", e);
                            EMPTY_STRING
                        }
                    });
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

#[tokio::main]
async fn main() {

    //----------------------------------------
    //   Structs and Variables definition
    //----------------------------------------


    //Instatiate the subscriber & file appender
    let file_appender = tracing_appender::rolling::hourly(LOG_PATH, "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();

    //Entering Main Loggin Level
    let main_span: span::Span = span!(Level::INFO,"Main");
    let _enter: span::Entered = main_span.enter();

    //Cloning the main span to set threads inside its scope
    let main_span_clone = main_span.clone();


    //MPSC - Initiate Thread message channels
    //Local messages
    let (local_message_tx, local_message_rx) = mpsc::channel();
    //Network messages received from peers
    let (network_message_tx, network_message_rx) = mpsc::channel();
    //Get model parameters from peer
    let (model_request_tx, model_request_rx) = mpsc::channel();
    //Reply from model
    let (model_reply_tx, model_reply_rx) = mpsc::channel();
    
    
    //Vector containing the selected MODEL and USER MESSAGE to be sent to model.
    let model_tuple: Vec<[String; 2]> = Vec::from([[EMPTY_STRING; 2]]); 

    //Prepare variable to be shared between threads
    let shared_model_tuple = Arc::new(Mutex::new(model_tuple));
    //Model & Message from web server
    let arc_web_write_model_tuple = Arc::clone(&shared_model_tuple);
    //Model & Message from local CLI
    let arc_cli_write_model_tuple = Arc::clone(&shared_model_tuple);
    //Model & Message from available model reply
    let arc_model_tuple_available = Arc::clone(&shared_model_tuple);
    
    
    //PEERS struct will be used to add Network peers information
    //Like PEERS address and model made available to public
    let my_self = Peers{address: MY_ADDRESS.to_string(), models: Vec::from(["Phi 3".to_string()])};

    //NODE struct contain machine side functions and config
    //NODE also contain Network related info 
    //As a PEERS Vector (which first item will be the node itself)
    let my_node: Node = Node{known_peers: Vec::from([my_self])};
    
    //Prepare variable to be shared between threads
    let shared_node = Arc::new(Mutex::new(my_node));
    //NODE instance used in local CLI
    let arc_cli_node = Arc::clone(&shared_node);
    //NODE instance used in network message thread
    let arc_net_node = Arc::clone(&shared_node);
    //NODE instance used in broadcast network message thread
    let arc_to_net_node = Arc::clone(&shared_node);
    //NODE instance to receive new peers from network
    let arc_web_net_node = Arc::clone(&shared_node);
    //NODE instance to perform automatic model addressing
    let arc_auto_model_node = Arc::clone(&shared_node);


    //Shared variable to store the MODEL MESSAGE (reply) and send to web server
    let shared_model_msg = Arc::new(Mutex::new(String::new()));
    //Receive message (reply) from model on main thread
    let arc_write_model_msg = Arc::clone(&shared_model_msg);
    //Display message (reply) from model on Web (server) page
    let arc_read_model_msg = Arc::clone(&shared_model_msg);


    //WEB form struct to store data get on post method
    //This data can be found in the index.html
    //Check for variables with same name and submit method
    let web_info = Vec::from([FormData{
        message: EMPTY_STRING,
        model: EMPTY_STRING,
        hidden_content: EMPTY_STRING,
    }]);

    //Prepare variable to be shared between threads
    let shared_web_info = Arc::new(Mutex::new(web_info));
    //Web info will be writen here in the Web Server
    let arc_write_web_info = Arc::clone(&shared_web_info);
    //Web info will be read in the net thread
    let arc_net_rw_web_info = Arc::clone(&shared_web_info);


    //BLOCK structure is intended to store the public message.
    //This message will be propagate into the network and
    //Nodes will be able to verify if they have the desired model
    //Once a match occurs, the node will start the request message function
    let blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Prepare variable to be shared between threads
    let shared_blocks = Arc::new(Mutex::new(blocks));
    //Append into the structure the blocks received from the network and Web Server
    let arc_web_net_write_blocks = Arc::clone(&shared_blocks);
    //Append into the structure the blocks inserted from CLI
    let arc_local_write_blocks = Arc::clone(&shared_blocks);
    //Read blocks messages on the main thread
    let arc_main_read_blocks = Arc::clone(&shared_blocks);
    //Read and Write blocks messages in the auto model message addressing section
    let arc_auto_model_wr_blocks = Arc::clone(&shared_blocks);


    //Initiate time measurement - for time triggered features
    let mut now = SystemTime::now();


    //------------------------------
    //   Thread Loops start here
    //------------------------------


    
    //Spawn thread for server initialization
    thread::spawn( move || main_span_clone.in_scope(move || network::net_init(network_message_tx, model_request_tx)));

    let local_users_join_handle = tokio::task::spawn_blocking(move || {local_users(local_message_tx)});

    //Network loop
    let net_web_join_handle = tokio::spawn( async move{            
        
        loop {

            // Check for new messages from the network thread
            let (net_msgs, net_peers) = handle_net_msg(&network_message_rx);
            //let net_msg: [String; 3] = handle_net_msg(&network_message_rx);

            for net_msg in net_msgs{
                if net_msg[0] != EMPTY_STRING {
    
                    let mut net_write_blocks = arc_web_net_write_blocks.lock().await;
    
                    if !net_write_blocks.message.contains(&net_msg){
    
                        //Call insert function to format and store in a block section
                        net_write_blocks.insert(net_msg.clone());
                    }
                }
                //else {
                    //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                //}
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            {
                let mut web_net_node = arc_web_net_node.lock().await;

                for peer in net_peers{

                    web_net_node.insert_peer(peer);

                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let mut net_web_rw_info = arc_net_rw_web_info.lock().await;

            if net_web_rw_info.len() > 1 {

                if net_web_rw_info[1].message != EMPTY_STRING {

                    let net_node = arc_net_node.lock().await;
                    let mut web_write_blocks = arc_web_net_write_blocks.lock().await;
    
                    let message: [String; 3] = [net_node.get_time_ns(), MY_ADDRESS.to_string(), net_web_rw_info[1].model.clone()];

                    web_write_blocks.insert(message.clone());

                    let mut web_write_model_message =  arc_web_write_model_tuple.lock().await;

                    web_write_model_message.push([net_web_rw_info[1].model.clone(), net_web_rw_info[1].message.clone()]);

                    net_web_rw_info.swap_remove(1);
    
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    


    let local_menu_join_handle = tokio::spawn( async move{

        loop{

            // Check for new messages from the input thread

            let (selection, (model, msg)) = handle_thread_msg(&local_message_rx);
            //println!("Menu local => {}", selection);

            if selection != 0 {

                match  selection {

                    1 => { //"1" => {

                        let mut local_write_blocks = arc_local_write_blocks.lock().await;

                        let mut cli_write_model_message = arc_cli_write_model_tuple.lock().await;

                        let cli_node = arc_cli_node.lock().await;

                        cli_write_model_message.push([model.clone(), msg.clone()]);

                        //Organize data to fit in the message format [current time, address, message text]
                        let message: [String; 3] = [cli_node.get_time_ns(), MY_ADDRESS.to_string(), String::from(model.trim())];

                        //Call insert function to format and store in a block section
                        local_write_blocks.insert(message.clone());

                    },

                    2 => { //"2" => {
                        //println!("\x1B[2J\x1B[1;1H");
                        
                        let cli_write_model_message = arc_cli_write_model_tuple.lock().await;
                        println!("-----------------------------");
                        println!("  !!!   Messages List   !!!");
                        println!("-----------------------------");
                        println!(" Messages => {:?}", cli_write_model_message);

                        println!("{}", MAIN_MENU);
                    },

                    3 => { //"3" => {

                        //println!("\x1B[2J\x1B[1;1H");

                        let cli_node = arc_cli_node.lock().await;
                        
                        let local_write_blocks = arc_local_write_blocks.lock().await;
                        println!("-----------------------------");
                        println!("  !!!  Updated blocks  !!!");
                        println!("-----------------------------");
                        println!(" Blocks => {:?}", local_write_blocks.message);
                        println!(" Peers => {:?}", cli_node.known_peers);

                        println!("{}", MAIN_MENU);
                    }

                    4 => { //"4" => {

                        //println!("\x1B[2J\x1B[1;1H");

                        //println!("Debug");
                        let owned_model = "Phi 3".to_string();
                        let local_write_blocks = {
                            let blocks = arc_local_write_blocks.lock().await;

                            let bb = blocks.message.clone();

                            bb
                        };

                        for msg in local_write_blocks.iter(){

                            if msg[2] == owned_model{

                                let remote_server_ip = msg[1].clone();
                                let model = msg[2].clone();

                                match TcpStream::connect(&remote_server_ip){
                                    Ok(_) => {

                                        tokio::task::spawn_blocking(move || {net::network::request_model_msg(remote_server_ip, model)});
                                    },
                                    Err(e) => {
                                        println!("Server not available, try later!");
                                        error!("Error while checking server connectivity => {}", e)
                                    },
                                }
                            } 
                            else {
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                println!("No messages for our server, try later!");
                            }
                        }

                        println!("{}", MAIN_MENU);
                    }
                    _ => (),
                }
            }
            else {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    let main_loop_join_handle = tokio::spawn(async move { loop{


        let received_model_msg = le_model(&model_reply_rx);

        if received_model_msg != EMPTY_STRING{
            let received_msg_len = received_model_msg.len();

            let mut write_msg = arc_write_model_msg.lock().await;

            if received_msg_len > 0{
                write_msg.clone_from(&received_model_msg);
            }
        }


        //Control of time triggered features
        match now.elapsed(){

            Ok(n) => {
                if n >= MINUTE{
                    info!("One minute");

                    let main_read_blocks = arc_main_read_blocks.lock().await;
                    let to_net_node = arc_to_net_node.lock().await;

                    let block_message = serde_json::to_string(&main_read_blocks.message).unwrap();

                    let net_msg = NetWorkMessage{
                        version: VERSION.to_string(),
                        time: to_net_node.get_time_ns(),
                        message: block_message,
                        peers: to_net_node.known_peers.clone(),
                        code: "00001".to_string(),
                    };

                    let msg = serde_json::to_string(&net_msg).unwrap();

                    //Spawn thread to propagate listening port to all network
                    tokio::spawn( async move {network::to_net(msg)});

                    now = SystemTime::now();
                }
            },
            Err(e) => error!("Error {}", e),
        }

        let model_tuple_available = arc_model_tuple_available.lock().await;
 
        match handle_model_available(&model_request_rx, model_tuple_available.clone(), model_reply_tx.clone()){
            Ok(n) => {

                if n != 0 {
                    info!("Message processed")
                }
            }
            Err(e) => {
                error!("Error while removing message from list => {}", e)
            }
        }

        thread::sleep(Duration::from_millis(1));
    }});


    let auto_model = tokio::spawn(async move{
        loop{

            
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            //println!("turnnnnnnnnnnnn");
            let (dest_ip, model) = {

                let mut r_ip = EMPTY_STRING;
                let mut r_model = EMPTY_STRING;

                println!("Debug 1");
                let auto_model_node = arc_auto_model_node.lock().await;

                let mut auto_model_wr_blocks = arc_auto_model_wr_blocks.lock().await;
                println!("Debug 2");
                for peer in auto_model_node.known_peers.clone(){
                    println!("Debug 3");
                    let mut i = 0;

                    for message in auto_model_wr_blocks.message.clone(){
                        println!("Debug 4");
                        for model in peer.models.clone(){
                            println!("Debug 5");
                            let dest_ip = peer.address.clone();
                            println!("Models => {} , {}", message[2], model);
                            println!("Debug 6");
                            if model == message[2]{
                                println!("Debug 7");
                                if model != EMPTY_STRING{
                                    //tokio::task::spawn_blocking(move || {net::network::request_model_msg(dest_ip, model)});
                                    //let eee = tokio::spawn(async move {net::network::request_model_msg(dest_ip, model)});
                                    //println!("Debugssss ip => {}, model=> {}", dest_ip, model);
                                    println!("iiiiiiiiiiiiiiii{}", i);
                                    //println!("Model => {}", message[2]);
                                    auto_model_wr_blocks.message.swap_remove(i);                               
                                    println!("Debug 8");
                                    //println!("Debugssss ip => {}, model=> {}", dest_ip, model);
                                    r_ip = dest_ip;
                                    r_model = model;
                                    
                                }                                                     
                            }
                        }
                        i += 1;                    
                    }                
                }

                (r_ip, r_model)

            };

            if model != EMPTY_STRING{
                //let _ = tokio::task::spawn_blocking(move || {net::network::request_model_msg(dest_ip, model)}).await;
                println!("Debugssss ip => {}, model=> {}", dest_ip, model);
            }
            
            //let _ = tokio::join!();
        
        }
    });


    let web_comm = AppState{
        model_message: arc_read_model_msg,
        web_info: arc_write_web_info,
    };

    let web_server_join_handle = tokio::spawn(async move {server(web_comm).await});


    //println!("End of main");

    let _ = tokio::join!(
        local_users_join_handle,
        net_web_join_handle,
        local_menu_join_handle,
        main_loop_join_handle,
        web_server_join_handle,
        auto_model
        );
}


//Async functions
async fn update_content(State(web): State<AppState>) -> Html<String> {
    
    let msg = web.model_message.lock().await;
    let h = Html(format!("<p> {} </p>", msg));
    h
}


async fn submit_form(State(arc_web): State<AppState>, Form(data): Form<FormData>) -> impl IntoResponse {
    println!("Received message: {:?}", data);
    println!("Received message: {}", data.message);
    println!("Received message: {}", data.model);
    println!("Received message: {}", data.hidden_content);

    let mut web = arc_web.web_info.lock().await;

    web.push(data);

    let file = tokio::fs::read_to_string("web/index.html").await.expect("Error while readin a file");
    Html(file)
}

async fn root() -> impl IntoResponse {

    let file = tokio::fs::read_to_string("web/index.html").await.expect("Error while readin a file");
    Html(file)
}

async fn server(web_comm: AppState){

    let app = Router::new()
    .route("/", get(root))
    .nest_service("/web", get_service(ServeDir::new("web")))
    .route("/update", get(update_content))
    .route("/submit", axum::routing::post(submit_form)) 
    .with_state(web_comm);

    let addr = "0.0.0.0:8680";

    println!("Listening on => {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}