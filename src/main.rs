use tokio::io::{BufWriter,BufReader,AsyncWriteExt,AsyncBufReadExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use core::time::Duration;

// Set arbitarily to 10ms before trying next thing.
// Idea behind this is a time_slice much like how an OS will time_split
const TIMEOUT_DURATION:Duration = Duration::from_millis(10);

// Using anyhow just so I don't need to deal with error transformation but also not use unwrap.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Using 8888 instead of 8080 since that was what was given in example
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let mut readers = Vec::new();
    let mut writers = Vec::new();
    loop {
        // First check if anyone wants to join
        if let Some(Ok((socket, socket_addr))) = timeout(TIMEOUT_DURATION,listener.accept()).await.ok(){
            let port = socket_addr.port();
            let (read, write) = socket.into_split();
            // Make bufreader and bufwriter so we don't need to deal with try_write
            let mut writer = BufWriter::new(write);
            let reader = BufReader::new(read);
            writer.write(&format!("LOGIN:{}\n",port).into_bytes()).await.ok();
            writer.flush().await.ok();
            readers.push((port,reader.lines()));
            writers.push((port,writer));
        }
        // Then for each person check their threads.
        for (reader_port,reader) in readers.iter_mut(){
            if let Ok(Ok(Some(line))) =  timeout(TIMEOUT_DURATION,reader.next_line()).await{
                for (writer_port,writer) in writers.iter_mut(){
                    if writer_port == reader_port{
                        writer.write(&format!("ACK:MESSAGE\n").into_bytes()).await.ok();
                        writer.flush().await.ok();
                    }else{
                       writer.write(&format!("Message:{} {}\n",reader_port,line).into_bytes()).await.ok();
                       writer.flush().await.ok();
                    }
                }
            }
        }
    }
}