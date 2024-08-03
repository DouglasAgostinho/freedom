/*
    ---------- Message code table version - 000.01 ----------
    ####1 - encryption handshake
    00000 - life beat message that broadcast listening port.
    00001 - Block propagation
*/
pub mod network{

    use std::thread;
    use base64::prelude::*;
    use std::sync::mpsc::Sender;    
    use std::io::{self, Read, Write}; 
    use std::net::{TcpListener, TcpStream};    
    use tracing::{instrument, info, error};
    use ring::agreement::{UnparsedPublicKey, X25519};
    use crate::crypt::crypt::{generate_own_keys, generate_shared_key, encrypt, decrypt}; 

    //----------Constants----------//

    //to use in String based variables
    const EMPTY_STRING: String = String::new();

    //Max number of peers
    const MAX_PEERS: u8 = 5;

    //Network buffer
    const BUFF: usize = 8192;
    pub const NET_BUFFER: [u8; BUFF] = [0; BUFF];

    //Constant Address & PORT
    pub const NET_PORT: &str = "6886";

    //Software version
    pub const VERSION: &str = "000_01";
    pub const VER_SIZE: usize = VERSION.len();

    //Message Code
    pub const TAIL_CODE: &str = "00000";
    pub const CODE_SIZE: usize = TAIL_CODE.len();
    
    
    #[instrument]
    fn handle_message(
        message: &String, 
        mode: &str, 
        tx: Sender<[String; 3]>, 
        income_stream: TcpStream, 
        model_tx: Sender<(TcpStream, String)>) -> bool {

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

                            "####1" => {
                                
                                let snd_model_msg = ser_msg.to_string();

                                if snd_model_msg != EMPTY_STRING {

                                    //Send net message to main thread
                                    if model_tx.send((income_stream, snd_model_msg)).is_err() {
                                        error!("Failed to send message to main thread.");
                                    }
                                    else {
                                        info!("Model Message sent OK")
                                    }
                                }
                                else {
                                    error!("Empty message from model request");
                                    
                                    ()
                                }
                            },

                            "####2" => {

                            }

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
    fn handle_client(
        mut stream: TcpStream, 
        tx: Sender<[String; 3]>, 
        model_tx: Sender<(TcpStream, String)>) {

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

            let received = String::from_utf8_lossy(&buf[..bytes_read]);

            let snd = tx.clone();
            let model_snd = model_tx.clone();

            let income_stream: TcpStream = match stream.try_clone(){
                Ok(s) => s,
                Err(e) => {
                    error!("Error while trying to clone stream => {}", e);
                    break
                }
            };

            if handle_message(&received.to_string(), "receive", snd, income_stream, model_snd) {

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
    pub fn net_init(tx: Sender<[String; 3]>, model_tx: Sender<(TcpStream, String)>){

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
            let model_snd = model_tx.clone();
            match stream {
                Err(e) => error!("Error found 0 {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_client(stream, snd, model_snd);
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

    fn client_connect(dest_addr: String) -> io::Result<TcpStream>{
        // Connect to the server
        let stream = TcpStream::connect(dest_addr)?;
        Ok(stream)
    }

    fn client_read(mut stream: TcpStream) -> (String, TcpStream){
        
        let mut buf: [u8; 8192] = NET_BUFFER;
        let bytes_read: usize = match stream.read(&mut buf){
            Ok(b) => b,
            Err(e) => {
                error!("Error while reading stream => {}", e);
                return (EMPTY_STRING, stream)
            }
        };

        let received: String = if bytes_read != 0 {
            String::from_utf8_lossy(&buf[..bytes_read]).to_string()
        }
        else {"!!!EMPTY_STRING!!!".to_string()};

        (received, stream)
    }

    fn client_write(mut stream: TcpStream, message: String) -> io::Result<TcpStream>{

        stream.write_all(message.as_bytes())?;

        Ok(stream)

    }

    /// Function responsible to perform message exchange securely by
    /// secure assynchronous key exchange and message encryption
    #[instrument]
    pub fn request_model_msg(dest_ip: String, model: String) -> io::Result<String> {
        
        //Generate own Ephemeral Keys
        let (pv_key, pb_key) = generate_own_keys();

        //Convert Public key to string
        let s_pb_key = BASE64_STANDARD.encode(pb_key);

        let to_snd_msg = (s_pb_key, model);

        let mut ser_snd_msg = serde_json::to_string(&to_snd_msg)?;

        ser_snd_msg.push_str("####1");    //####1 - code for encryption handshake
        ser_snd_msg.push_str(VERSION);    //Insert software version in message tail

        
        let client_stream = client_connect(dest_ip)?;
        
        let client_stream = client_write(client_stream, ser_snd_msg)?;
        
        let (ser_crypto, client_stream) = client_read(client_stream);

        //Send request for model message and Public Key
        //Received message will come as a serialized tuple (encrypted message, client public key)
        let received_crypto :(String, Vec<u8>) = serde_json::from_str(&ser_crypto)?;

        //Decoding client public key
        let decoded_pb_key = match BASE64_STANDARD.decode(received_crypto.0){
            Ok(d) => d,
            Err(e) => {
                error!("Error found while decoding pb Key => {}", e);
                Vec::new()
            }
        };

        // Create an `UnparsedPublicKey` from the bytes
        let cl_pub_key = UnparsedPublicKey::new(&X25519, decoded_pb_key.clone());

        //Generate shared synchronous key
        let shared_key = generate_shared_key(pv_key, cl_pub_key);

        //Decrypt received message
        let msg = decrypt(shared_key, received_crypto.1);

        let model_address = "192.168.191.1:8687".to_string();

        // Connect to the model
        let model_stream = client_connect(model_address)?;

        // Send data to the server
        let model_stream = client_write(model_stream, msg)?;

        loop{            

            let (model_msg, _) = client_read(model_stream.try_clone()?);

            if model_msg == "!!!EMPTY_STRING!!!".to_string() {
                break;
            }        
            
            let crypt_msg = encrypt(shared_key, model_msg.clone());

            let crypt_ser = BASE64_STANDARD.encode(crypt_msg);

            match client_write(client_stream.try_clone()?, crypt_ser){
                Ok(_) => (),
                Err(e) => println!("Error found while sendind model msg => {}", e),
            }
        }

        Ok(EMPTY_STRING)
    }

    #[instrument]
    pub fn send_model_msg(encoded_key: String, message: String, mut income_stream: TcpStream, tx: Sender<String>) -> io::Result<String> {

        //Decoding received public key
        let decoded_pb_key = match BASE64_STANDARD.decode(encoded_key){
            Ok(d) => d,
            Err(e) => {
                error!("Error found while decoding pb Key => {}", e);
                Vec::new()
            }
        };

        // Create an `UnparsedPublicKey` from the bytes
        let server_pb_key = UnparsedPublicKey::new(&X25519, decoded_pb_key.clone());

        //Generate own Ephemeral Keys
        let (pv_key, pb_key) = generate_own_keys();

        //Generate shared synchronous key
        let shared_key = generate_shared_key(pv_key, server_pb_key);

        //Encrypt message
        let crypt_msg = encrypt(shared_key, message);

        //Encoding own public key
        let encoded_my_pb = BASE64_STANDARD.encode(pb_key);

        //Formating message tuple (encrypted message, client public key)
        let crypt_tuple: (String, Vec<u8>)= (encoded_my_pb, crypt_msg);

        //Serialize formated message
        let ser_crypt_msg = serde_json::to_string(&crypt_tuple)?;

        //Repply to client serialized message
        income_stream.write_all(ser_crypt_msg.as_bytes())?;

        let mut model_message = EMPTY_STRING;
        
        loop{            

            let (ser_crypt_msg, _) = client_read(income_stream.try_clone()?);

            if ser_crypt_msg == "!!!EMPTY_STRING!!!".to_string() {
                break;
            }

            let crypt_msg: Vec<u8> = match BASE64_STANDARD.decode(ser_crypt_msg.clone()){
                Ok(v) => v,
                Err(e) => {
                    println!("err {}", e);
                    continue
                    
                },
            };

            let model_msg = decrypt(shared_key, crypt_msg);

            model_message.push_str(&model_msg);

                        
            match tx.send(model_message.clone()){            
                Ok(t) => {
                    t},
                Err(e) => {
                    error!("Failed to send input to main thread => {}", e);            
                }
            }
        }

        //println!("{}", MAIN_MENU);

        Ok(EMPTY_STRING)
    }

}