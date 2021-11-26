use std::io::{ErrorKind,Read,Write};
use std::net::TcpStream;
use std::sync::mpsc::{self,TryRecvError};
use std::thread;
use std::time::Duration;


const LOCAL_HOST : &str = "159.75.121.123:9999";//服务端地址
const MESSAGE_SIZE : usize = 1024;//缓冲区大小

fn main() {
    let mut client = TcpStream::connect(LOCAL_HOST)
        .expect("connect failed");
    client
        .set_nonblocking(true)
        .expect("failed to nonblocking");
    let (tx,rx) = mpsc::channel::<String>();
    std::thread::spawn(move || loop{
        let mut buff = vec![0;MESSAGE_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_)=>{
                let msg = buff
                    .into_iter()
                    .take_while(|&x|x!=0)
                    .collect::<Vec<_>>();
                let msg_string = String::from_utf8(msg.clone())
                    .unwrap();
                println!("{msg_string:>width$}",msg_string=msg_string,width=50);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_)=> {
                println!("server down");
                break;
            }
        }
        match rx.try_recv() {
            Ok(msg)=>{
                let mut buff = msg
                    .clone()
                    .into_bytes();
                buff
                    .resize(MESSAGE_SIZE,0);
                client
                    .write_all(&buff)
                    .expect("Writing to socket failed");
                // println!("msg send: {:?}",msg);
            },
            Err(TryRecvError::Empty) =>(),
            Err(TryRecvError::Disconnected) => break
        }
        thread::sleep(Duration::from_millis(100));
    });
    println!("msg: ");
    loop {
        let mut buff = String::new();
        std::io::stdin()
            .read_line(&mut buff)
            .expect("readingg from stdin failed");
        let msg = buff.trim().to_string();
        if msg == "/exit" || tx.send(msg).is_err(){break}
    }
    println!("offline");
}
