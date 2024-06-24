pub mod network{
    
    
    use std::thread;        
    use std::io::{self,Read, Write};
    use std::sync::mpsc::Sender;
    use std::net::{TcpListener, TcpStream};    
    //use sha2::digest::consts::False;
    

    //----------Constants----------//

    //to use in String based variables
    const EMPTY_STRING: String = String::new();

    //Max number of peers
    const MAX_PEERS: u8 = 5;

    //Network buffer
    const NET_BUFFER: [u8; 2048] = [0; 2048];

    
    //Constant Address & PORT
    pub const NET_PORT: &str = "6886";
    //pub const PORT_SIZE: usize = NET_PORT.len();

    //Software version
    //pub const VERSION: &str = "000_01";
    //pub const VER_SIZE: usize = VERSION.len();

    //Message Code
    //pub const INIT_CODE: &str = "00000";
    //pub const CODE_SIZE: usize = INIT_CODE.len();
    
    

    fn handle_message(message: &String, mode: &str, tx: Sender<[String; 3]>) -> bool{
        //Function to treat incoming / outgoing messages        

        let pure_msg = message.clone();
        
        match mode {

            "send" => {false},

            "receive" => {
                let msg = message.trim();
                
                match msg {                    

                    "[!]_stream_[!]" => true, 
                                   
                    _ => {
                        println!("Received: {}", message);    
                        println!("Pure: {}", pure_msg);    
                        
                        let mut net_message :Vec<[String; 3]> = serde_json::from_str(&message).expect("Error");
                        
                        let msg_code = "00001";
                        
                        match msg_code {

                            "00000" => println!("Message -> {}", msg),

                            "00001" => { //Block received

                                loop{

                                    //let mut user_msg: [String; 3] = [EMPTY_STRING; 3];
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
                                            eprintln!("Failed to send message to main thread.");                                    
                                        }                                        
                                    }
                                    else {
                                        break;
                                    }                      
                                }                                
                            }

                            "00002" => println!("Received message => {}", message),
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


    fn handle_client(mut stream: TcpStream, tx: Sender<[String; 3]>) {        

        let income_addr = stream.peer_addr().expect("Error");
        
        println!("Incoming connection from {}", income_addr);
        let mut buf = NET_BUFFER;
        

        loop {
                        
            let bytes_read = stream.read(&mut buf).expect("Error");            
            if bytes_read == 0 {break}
            
            let received = String::from_utf8_lossy(&buf[..bytes_read]);
        
            let snd = tx.clone();

            if handle_message(&received.to_string(), "receive", snd) {
                                                     
                stream.write_all("ready_to_receive".as_bytes()).expect("error");
                // Receive data continuously from the server
                let mut buffer = NET_BUFFER;                    
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Connection closed by server");
                            break;
                        },
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buffer[0..n]);
                            print!("{}", msg);      //Uses print! to not insert /n after each received data                                                            
                            io::stdout().flush().expect("error");  // Ensure immediate output
                        },
                        Err(e) => {
                            println!("Failed to receive message: {}", e);
                            break;
                        }
                    }
                }                    
            } 
            else {
                println!("Connection closed by server");
                break;                
            }           
        }
    }


    pub fn net_init(tx: Sender<[String; 3]>){
        //Composing IP address with received port
        let mut addr = String::from("0.0.0.0:");
        addr.push_str(NET_PORT);

        //Set system to listen
        let listener = TcpListener::bind(addr).expect("Could not bind");
        println!("Server initialized...");
        
        for stream in listener.incoming(){
            let snd = tx.clone();
            match stream {
                Err(e) => println!("Error found 0 {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
        
                        handle_client(stream, snd);
                    });
                }
            }
        }
    }        
    
    /// Broadcast message to all Network    
        pub fn to_net(send_what: String) {                

        for n in 1..MAX_PEERS {         

            //let msg = message.clone();    
            let msg = send_what.clone();    

            //Loop through all address
            let address = format!("192.168.191.{}:6886", n);

            //call client function to send message
            thread::spawn(move || match client(msg, &address, "simple"){

                Ok(_) => (),
                Err(e) => println!("On host {} Error found {}",address, e),
            });
        }
    }    


    
    fn client(message: String, address: &str, mode: &str)-> io::Result<()> {
        match mode {
            "simple" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            "serialized" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;

                //println!("Pre Serde {:?}", message);

                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                Ok(())
            },
            "test" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            _ => Ok(()),
        }
    }
}