pub mod network{

    use std::net::{TcpListener, TcpStream};
    use std::thread;
    //use std::io::{self,Read, Write, Error};
    use std::io::{self,Read, Write};


    fn handle_message(message: &String, mode: &str) -> u8{
        //Function to treat incoming / outgoing messages

        match mode {

            "encrypt" => {0},

            "decrypt_llm" => {0},

            "interpret" => {
                match message {

                    

                    _ => 0,
                }
            },

            "test" => {
                println!("Received: {}", message);
                254
            },
            _ => 0,
        }
    }


    fn handle_client(mut stream: TcpStream, mode: &str) {
        //fn handle_client(mut stream: TcpStream) -> Result<(), Error>{

        //println!("Incoming connection from {}", stream.peer_addr()?);   
        println!("Incoming connection from {}", stream.peer_addr().expect("Error"));
        let mut buf = [0; 512];

        loop {
            
            //let bytes_read = stream.read(&mut buf)?;
            let bytes_read = stream.read(&mut buf).expect("Error");
            //if bytes_read == 0 {return Ok(())}
            if bytes_read == 0 {break}
            let received = String::from_utf8_lossy(&buf[..bytes_read]);
            match handle_message(&received.to_string(), mode) {

                1 => {
                    //Receive blocks from server
                    let mut buffer = [0; 1024];
                    //let bytes_read = stream.read(&mut buffer)?;
                    let bytes_read = stream.read(&mut buffer).expect("Error");
                    let received = String::from_utf8_lossy(&buffer[..bytes_read]);
                    //println!("Received: {}", received);
                },
                2 => {
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
                                //io::stdout().flush()?;  // Ensure immediate output
                                io::stdout().flush().expect("error");  // Ensure immediate output
                            },
                            Err(e) => {
                                println!("Failed to receive message: {}", e);
                                break;
                            }
                        }
                    }
                    //Ok(())
                },

                _ => {},
            }
            
        }
    }


    pub fn net_init(){

        let listener = TcpListener::bind("0.0.0.0:6886").expect("Could not bind");
        println!("Server initialized...");

        for stream in listener.incoming(){
            match stream {
                Err(e) => println!("Error found {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        //handle_client(stream).unwrap_or_else(|error| println!("Error {:?}", error));
                        handle_client(stream, "interpret");
                    });
                }
            }
        }
    }


    pub fn client(message: &str, address: &str, mode: i32)-> io::Result<()> {
        match mode {
            1 => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            2 => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;

                //println!("Pre Serde {:?}", message);

                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                Ok(())
            },
            3 => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            _ => Ok(()),
        }
    }
}