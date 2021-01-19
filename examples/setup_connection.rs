// TODO:
// 1. [x] - Import the stratumv2 crate
// 2. Create x2 threads - Mining Pool (server) and Mining Device (client)
//   - [x] Set open local ports for server and client
//   - [x] Create a TCP connection between server and client
// 3. Mining Device sends a `SetupConnection`
//   - [x] Serialize the message and send to the mining pool
// 4. Mining Pool responds with a `SetupConnection.Success`
//   - [] Parse the message and lookup the type of message
//   - [] Send a SetupConnectionSuccess
// 5. Client Device receives SetupConnection.Success
//   - [] Parse the message
// 6. Exit the program after all messages have been sent out
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use stratumv2::common::messages::SetupConnection;
use stratumv2::common::Framable;
use stratumv2::common::Protocol;
use stratumv2::mining;

// TODO: I think it makes more sense for imports to follow ::mining::flags::SetupConnectionFlags,
use stratumv2::mining::SetupConnectionFlags;

fn main() {
    let (tx, rx) = mpsc::channel();
    // let tx1 = mpsc::Sender::clone(&tx);

    // Mining Pool (server) thread.
    let mining_pool_url = "127.0.0.1:9545";
    thread::spawn(move || {
        let listener = TcpListener::bind(mining_pool_url).unwrap();

        // TODO: I suppose I can keep parsing the message to understand the
        // variable length of the payload.
        let mut buffer = [0u8; 1024];
        for stream in listener.incoming() {
            stream.unwrap().read(&mut buffer).unwrap();
            // println!("First byte in buffer: {}", buffer[0]);
            // TODO: Match the first byte in the buffer.
            //
            // TODO: Find the payload_length
            // Deserialize into the object Some(x)
            // or None.
            let msg_type = buffer[2];
            let msg = match msg_type {
                0x00 => {
                    let payload_length: &[u8] = &buffer[3..6];
                    println!("DEBUG: payload_lenght: {:?}", payload_length);

                    // TODO: Move into a macro?
                    let mut x = 0x00;
                    for byte in payload_length.iter() {
                        x |= byte
                    }
                    let result = x as u32;
                    println!("DEBUG: payload length: {}", result);

                    let payload = &buffer[6..result as usize];
                    println!("DEBUG only payload: {:?}", payload);
                    println!("DEBUG length of actual payload: {:?}", payload.len());

                    // TODO: Need to redesign SetupConnection where the caller
                    // DOESN'T need to say "hey I'm using mining flags"
                    let setup_connection =
                        SetupConnection::<mining::SetupConnectionFlags>::deserialize(payload)
                            .unwrap();
                    println!("DEBUG after calling deserialize");
                    Some(setup_connection)
                }
                _ => None,
            };
            tx.send(buffer.clone()).unwrap();
        }
    });

    // Mining device (client) thread.
    let mining_device_url = "127.0.0.1:8545";
    thread::spawn(move || {
        let setup_connection_msg = SetupConnection::new(
            Protocol::Mining,
            2,
            2,
            &[SetupConnectionFlags::RequiresStandardJobs],
            "127.0.0.1",
            8545,
            "Slushpool",
            "S91 13.5",
            "braiins-os-2018-09-22-1-hash",
            "some-device-uuid",
        )
        .unwrap();

        let mut stream = TcpStream::connect(mining_pool_url).unwrap();
        setup_connection_msg.frame(&mut stream).unwrap();

        // Client listens to responses from mining pool.
        let listener = TcpListener::bind(mining_device_url).unwrap();

        let mut buffer = [0u8; 1024];
        for stream in listener.incoming() {
            stream.unwrap().read(&mut buffer).unwrap();
        }
    });

    for recevied in rx {
        println!("Received: {:?}", recevied);
    }
}
