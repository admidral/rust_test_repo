use tokio::io::{BufWriter,BufReader,AsyncWriteExt,AsyncBufReadExt};
use tokio::net::{TcpListener,tcp::{OwnedReadHalf,OwnedWriteHalf}};
use tokio::sync::broadcast::{Sender,Receiver};
#[derive(Clone,Debug)]
struct Message{
    pub port: u16,
    pub message: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Using 8888 instead of 8080 since that was what was given in example
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    //Assuming the send pressure isn't super high. 1000 is a very large lag factor.
    let sender = Sender::new(1000);
    loop {
        let (socket, socket_addr) = listener.accept().await?;
        // Use IP port and ID;
        let port = socket_addr.port();
        let (read, write) = socket.into_split();
        // Immediately respond with login
        let writer = BufWriter::new(write);
        let reader = BufReader::new(read);
        tokio::spawn(read_side(port,reader,sender.clone()));
        tokio::spawn(write_side(port,writer,sender.subscribe()));
    }
}

async fn read_side(port: u16,listener:BufReader<OwnedReadHalf>,tx: Sender<Message>)-> anyhow::Result<()>{
    let mut lines = listener.lines();
    while let Some(line) = lines.next_line().await? {
        let message = Message{
            port,
            message: line
        };
        tx.send(message)?;
    }
    Ok(())
}



async fn write_side(port:u16,mut writer: BufWriter<OwnedWriteHalf>,mut rx:Receiver<Message>)-> anyhow::Result<()>{
    writer.write(&format!("LOGIN:{}\n",port).into_bytes()).await?;
    writer.flush().await?;
    loop{
        let message = rx.recv().await?;
        if message.port == port{
            writer.write(&format!("ACK:MESSAGE\n").into_bytes()).await?;
            writer.flush().await?;
        }else{
           writer.write(&format!("Message:{} {}\n",message.port,message.message).into_bytes()).await?;
           writer.flush().await?;
        }
    }
}