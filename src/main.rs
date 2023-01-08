use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

const LOCAL_SERVER: &str = "127.0.0.1:8888";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(LOCAL_SERVER).await?;

    // 对所有客户端进行广播
    // 在收到消息时，把消息地址传入channel中
    let (tx, _) = broadcast::channel(12);


    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("{} connected.", addr);
        let tx = tx.clone();
        let mut rx = tx.subscribe();


        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut msg = String::new();
            loop {
                // 客户端给服务端发消息，然后服务端对消息进行广播
                tokio::select! {
                    // 从 socket 里读
                    result = reader.read_line(&mut msg) => {
                        // 什么都没读到
                        if result.unwrap() == 0 {
                            break;
                        }
                        println!("{}", msg);

                        // // (消息, 消息发送者)
                        tx.send((msg.clone(), addr)).unwrap();
                        msg.clear();
                    }

                    // 从 channel 里读
                    result = rx.recv() => {
                        let (msg_str, other_address) = result.unwrap();

                        // 接收消息时，剔除掉发送消息的客户端
                        // TODO: 应该把发送消息的客户端的ip也打包广播出去
                        println!("{} send {} to {}", addr, &msg_str.trim(), other_address);
                        if addr != other_address {
                            writer.write_all(msg_str.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}