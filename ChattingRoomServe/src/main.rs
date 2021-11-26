use std::net::TcpListener;
use std::sync::mpsc::{self};
use std::io::{ErrorKind, Read, Write};
use std::thread;
use std::time::Duration;
use std::str;

const LOCAL_HOST : &str = "0.0.0.0:9999";//服务端地址
const MESSAGE_SIZE : usize = 1024;//缓冲区大小


fn main() {
    //监听相应端口
    let server = TcpListener::bind(LOCAL_HOST)
        .expect("listen failed");
    server
        .set_nonblocking(true)//设置成非阻塞
        .expect("failed to nonblocking");
    let mut clients = vec![];
    let (tx,rx) = mpsc::channel::<String>();
    loop{
        if let Ok((mut socket , addr)) = server.accept(){
            println!("{} connected", addr);
            //将用户加入channel
            clients
                .push(socket.try_clone()
                    .expect("failed to clone client"));
            let tx = tx.clone();
            //创建子线程
            thread::spawn(move || loop {
                //接受信息，读取buff内容
                let mut buff: Vec<u8> = vec![0;MESSAGE_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(..) => {
                        let msg=buff
                            .into_iter()
                            .take_while(|&x|x!=0)
                            .collect::<Vec<_>>();
                        let msg_string = String::from_utf8(msg)
                            .expect("Invalid utf8 message");
                        //打印
                        println!("{}:{:?}",addr, msg_string);
                        tx
                            .send(msg_string)
                            .expect("Failed message send");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock=>(),
                    Err(..) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            });
        }
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client|{
                    let mut buff = msg.clone()
                        .into_bytes();
                    buff
                        .resize(MESSAGE_SIZE.into(),0);
                    client
                        .write_all(&buff).map(|_|client).ok()
                }).collect::<Vec<_>>();
        }
        thread::sleep(Duration::from_millis(100));
    }
}