pub mod network{
    
    //use std::os::unix::net::SocketAddr;
    use std::thread;        
    use std::io::{self,Read, Write};
    use std::net::{TcpListener, TcpStream};    
    //use sha2::digest::consts::False;
    

    //----------Constants----------//

    //to use in String based variables
    //const EMPTY_STRING: String = String::new();

    //Max number of peers
    const MAX_PEERS: u8 = 5;

    
    //Constant Address & PORT
    pub const NET_PORT: &str = "6886";
    pub const PORT_SIZE: usize = NET_PORT.len();

    //Software version
    pub const VERSION: &str = "000.01";
    pub const VER_SIZE: usize = VERSION.len() + 1;

    //Message Code
    pub const INIT_CODE: &str = "00000";
    pub const CODE_SIZE: usize = INIT_CODE.len();
    

    fn handle_message(message: &String, mode: &str) -> bool{
        //Function to treat incoming / outgoing messages        
        match mode {

            "send" => {false},

            "receive" => {
                let msg = message.trim();
                
                match msg {                    

                    "[!]_stream_[!]" => true, 
                                   
                    _ => {
                        println!("Received: {}", message);    
                        
                        let len = message.len();
                        let client_ver = &message[len - VER_SIZE .. len];
                        let msg_code = &message[len - CODE_SIZE - VER_SIZE .. len - VER_SIZE];    
                        let client_port = &message[len - PORT_SIZE - CODE_SIZE - VER_SIZE .. len - CODE_SIZE - VER_SIZE];  
                        let msg = &message[0 .. len - PORT_SIZE - CODE_SIZE - VER_SIZE];

                        println!("version -> {} & code -> {}", client_ver, msg_code);
                        println!(" client port -> {} & msg -> {}", client_port, msg );

                        match msg_code {

                            "00000" => println!("Message -> {}", msg),

                            "00002" => println!("Received Block => {}", message),
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


    fn handle_client(mut stream: TcpStream) {        

        let income_addr = stream.peer_addr().expect("Error");
        //println!("Incoming connection from {}", stream.peer_addr()?);   
        println!("Incoming connection from {}", income_addr);
        let mut buf = [0; 1024];

        loop {
                        
            let bytes_read = stream.read(&mut buf).expect("Error");            
            if bytes_read == 0 {break}
            
            let received = String::from_utf8_lossy(&buf[..bytes_read]);

            if handle_message(&received.to_string(), "receive") {                                
                                                     
                stream.write_all("ready_to_receive".as_bytes()).expect("error");
                // Receive data continuously from the server
                let mut buffer = [0; 1024];                    
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


    pub fn net_init(){
        //Composing IP address with received port
        let mut addr = String::from("0.0.0.0:");
        addr.push_str(NET_PORT);

        //Set system to listen
        let listener = TcpListener::bind(addr).expect("Could not bind");
        println!("Server initialized...");
        
        //thread::spawn( || to_net("!who_is_alive!"));

        for stream in listener.incoming(){
            match stream {
                Err(e) => println!("Error found {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        //handle_client(stream).unwrap_or_else(|error| println!("Error {:?}", error));
                        handle_client(stream);
                    });
                }
            }
        }
    }        
    
    /// Broadcast message to all Network    
    pub fn to_net(send_what: &str) {                

        let message = serde_json::to_string(send_what).expect("Error");

        //message.push_str(VERSION);

        //let msg: &str = &message[ .. ];

        for n in 1..MAX_PEERS {         

            let msg = message.clone();    

            //Loop through all address
            let address = format!("192.168.191.{}:6886", n);

            //call client function to send message
            thread::spawn(move || match client(&msg, &address, "simple"){
                Ok(_) => (),
                Err(e) => println!("On host {} Error found {}",address, e),
            });
        }
    }    


    fn client(message: &str, address: &str, mode: &str)-> io::Result<()> {
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