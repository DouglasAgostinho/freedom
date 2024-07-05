/*
    ---------- Message code table version - 000.01 ----------
    ##### - encryption handshake
    00000 - life beat message that broadcast listening port.
    00001 - Block propagation
*/
pub mod network{

    use std::thread;
    use std::io::{self,Read, Write};
    use std::sync::mpsc::Sender;
    use std::net::{TcpListener, TcpStream};
    use base64::prelude::*;
    use tracing::{instrument, info, error};
    use ring::agreement::{UnparsedPublicKey, X25519};
    use crate::crypt::crypt::{generate_own_keys, generate_shared_key, encrypt, decrypt}; 

    //----------Constants----------//

    //to use in String based variables
    const EMPTY_STRING: String = String::new();

    //Max number of peers
    const MAX_PEERS: u8 = 5;

    //Network buffer
    const NET_BUFFER: [u8; 2048] = [0; 2048];

    //Constant Address & PORT
    pub const NET_PORT: &str = "6886";

    //Software version
    pub const VERSION: &str = "000_01";
    pub const VER_SIZE: usize = VERSION.len();

    //Message Code
    pub const TAIL_CODE: &str = "00000";
    pub const CODE_SIZE: usize = TAIL_CODE.len();

    #[instrument]
    fn handle_message(message: &String, mode: &str, tx: Sender<[String; 3]>, income_stream: TcpStream) -> bool{

        //Function to treat incoming / outgoing messages
        let msg_len = message.len();

        let ser_msg = &message[ .. msg_len - CODE_SIZE - VER_SIZE];

        match mode {

            "send" => {false},

            "receive" => {
                let msg = message.trim();

                match msg {

                    "[!]_stream_[!]" => true,

                    _ => {
                        info!("Received: {}", message);

                        let msg_code = &message[msg_len - CODE_SIZE - VER_SIZE .. msg_len - VER_SIZE];

                        match msg_code {

                            "#####" => {

                                send_model_msg(ser_msg.to_string(), income_stream);
                            },

                            "00000" => println!("Message -> {}", msg),

                            "00001" => { //Block received
                                let mut net_message :Vec<[String; 3]> = match serde_json::from_str(ser_msg){

                                    Ok(msg) => msg,
                                    Err(e) => {
                                        error!("Error while deserializing Net Message => {}", e);
                                        Vec::from([[EMPTY_STRING; 3]])
                                    }
                                };

                                loop{

                                    let user_msg: [String; 3];

                                    if let Some(_) =  net_message.get(1){

                                        user_msg = net_message.swap_remove(1);

                                    }
                                    else{
                                        user_msg = [EMPTY_STRING; 3];
                                    }
                                    
                                    if user_msg[0] != EMPTY_STRING {

                                        //Send net message to main thread
                                        if tx.send(user_msg).is_err() {
                                            error!("Failed to send message to main thread.");
                                        }
                                    }
                                    else {
                                        break;
                                    }
                                }
                            }

                            _ => (),
                        }
                        false //to_do Will return decrypted message
                    },
                }
            },

            "test" => {
                println!("Received: {}", message);
                false
            },

            _ => false,
        }
    }

    #[instrument]
    fn handle_client(mut stream: TcpStream, tx: Sender<[String; 3]>) {

        let income_addr = match stream.peer_addr(){
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to retrieve incoming connectiong address => {}", e);
                return
            }
        };
        
        info!("Incoming connection from {}", income_addr);
        let mut buf = NET_BUFFER;
        
        loop {

            let bytes_read = match stream.read(&mut buf){
                Ok(0) => {
                    info!("Connection closed by server");
                    break;
                },
                Ok(b) => b,
                Err(e) => {
                    error!("Error while reading stream => {}", e);
                    break;
                }
            };

            //if bytes_read == 0 {break}

            let received = String::from_utf8_lossy(&buf[..bytes_read]);

            let snd = tx.clone();

            let income_stream: TcpStream = match stream.try_clone(){
                Ok(s) => s,
                Err(e) => {
                    error!("Error while trying to clone stream => {}", e);
                    break
                }
            };

            if handle_message(&received.to_string(), "receive", snd, income_stream) {

                //Repply to client that server is ready to receive stream
                match stream.write_all("ready_to_receive".as_bytes()){
                    Ok(s) => s,
                    Err(e) => {
                        error!("Error while trying to send network message => {}", e);
                        return
                    }
                }

                //Create buffer to receive data
                let mut buffer = NET_BUFFER;

                // Receive data continuously from the server
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            info!("Connection closed by server");
                            break;
                        },
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buffer[0..n]);
                            print!("{}", msg);      //Uses print! to not insert /n after each received data
                            // Ensure immediate output
                            match io::stdout().flush(){
                                Ok(n) => n,
                                Err(e) => {
                                    error!("Error while flushing Std output => {}", e);
                                    break
                                }
                            }  
                        },
                        Err(e) => {
                            error!("Failed to receive message: {}", e);
                            break;
                        }
                    }
                }
            }
            else {
                info!("Connection closed by server");
                break;
            }
        }
    }

    #[instrument]
    pub fn net_init(tx: Sender<[String; 3]>){

        //Composing IP address with received port
        let mut addr = String::from("0.0.0.0:");
        addr.push_str(NET_PORT);

        //Set system to listen
        let listener = match TcpListener::bind(addr){
            Ok(l) => l,
            Err(e) => {
                error!("Error while binding address => {}", e);
                return
            }
        };

        info!("Server initialized...");
        
        //Create a thread for each received connection
        for stream in listener.incoming(){
            let snd = tx.clone();
            match stream {
                Err(e) => error!("Error found 0 {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_client(stream, snd);
                    });
                }
            }
        }
    }
    
    /// Broadcast message to all Network
    #[instrument]
    pub fn to_net(send_what: String) {

        for n in 1..MAX_PEERS {
   
            let msg = send_what.clone();

            //Loop through all address
            let address = format!("192.168.191.{}:6886", n);

            //call client function to send message
            thread::spawn(move || match client(msg, &address, "simple"){

                Ok(_) => (),
                Err(e) => error!("On host {} Error found {}",address, e),
            });
        }
    }    


    fn client(message: String, address: &str, mode: &str)-> io::Result<String> {
        match mode {
            "simple" => {
                println!("done {}", message);
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;
                Ok(EMPTY_STRING)
            },
            "serialized" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;

                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                Ok(EMPTY_STRING)
            },
            "test" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(message.as_bytes())?;
                Ok(EMPTY_STRING)
            },
            "model_msg" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;

                stream.flush()?;

                let mut buf = NET_BUFFER;
                let bytes_read = match stream.read(&mut buf){
                    Ok(b) => b,
                    Err(e) => {
                        error!("Error while reading stream => {}", e);
                        return Ok(EMPTY_STRING)
                    }
                };
                
                let received = if bytes_read != 0 {
                    String::from_utf8_lossy(&buf[..bytes_read]).to_string()
                }
                else {EMPTY_STRING};

                println!("received client {}", received);

                Ok(received)
            },
            _ => Ok(EMPTY_STRING),
        }
    }

    /// Function responsible to perform message exchange securely by
    /// secure assynchronous key exchange and message encryption
    pub fn request_model_msg(dest_ip: String){
        
        //Generate own Ephemeral Keys
        let (pv_key, pb_key) = generate_own_keys();

        //Convert Public key to string
        let mut s_pb_key = BASE64_STANDARD.encode(pb_key);

        s_pb_key.push_str("#####");    //##### - code for encryption handshake
        s_pb_key.push_str(VERSION);    //Insert software version in message tail


        //Send request for model message and Public Key
        let ser_crypto: String = match client(s_pb_key, &dest_ip, "model_msg"){
            Ok(s) => s,
            Err(e) => {
                error!("Error while requesting client message => {}", e);
                return
            }
        };

        println!(" rec {}", ser_crypto);

        //Received message will come as a serialized tuple (encrypted message, client public key)
        let received_crypto :(String, Vec<u8>) = serde_json::from_str(&ser_crypto).expect("Error");

        //Decoding client public key
        let decoded_pb_key = BASE64_STANDARD.decode(received_crypto.0).expect("error");

        // Create an `UnparsedPublicKey` from the bytes
        let cl_pub_key = UnparsedPublicKey::new(&X25519, decoded_pb_key.clone());

        //Generate shared synchronous key
        let shared_key = generate_shared_key(pv_key, cl_pub_key);

        //Decrypt received message
        let msg = decrypt(shared_key, received_crypto.1);

        println!("Decrypted {}", msg);
    }

    pub fn send_model_msg(encoded_key: String, mut income_stream: TcpStream){

        //Decoding received public key
        let decoded_pb_key = BASE64_STANDARD.decode(encoded_key).expect("error");

        // Create an `UnparsedPublicKey` from the bytes
        let server_pb_key = UnparsedPublicKey::new(&X25519, decoded_pb_key.clone());

        //Generate own Ephemeral Keys
        let (pv_key, pb_key) = generate_own_keys();

        //Generate shared synchronous key
        let shared_key = generate_shared_key(pv_key, server_pb_key);

        //Encrypt message
        let crypt_msg = encrypt(shared_key, "LapTop secret".to_string());

        //Encoding own public key
        let encoded_my_pb = BASE64_STANDARD.encode(pb_key);

        //Formating message tuple (encrypted message, client public key)
        let crypt_tuple: (String, Vec<u8>)= (encoded_my_pb, crypt_msg);

        //Serialize formated message
        let ser_crypt_msg = serde_json::to_string(&crypt_tuple).expect("error");

        //Repply to client serialized message
        income_stream.write_all(ser_crypt_msg.as_bytes()).expect("error");
    }

    ///Function to receive or send model repply
    pub fn _process_model_msg(){        

    }
    
}