use tokio::io::{BufWriter,BufReader,AsyncWriteExt,AsyncBufReadExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use core::time::Duration;
use std::collections::HashMap;
// Set arbitarily to 10ms before trying next thing.
// Idea behind this is a time_slice much like how an OS will time_split
const TIMEOUT_DURATION:Duration = Duration::from_millis(10);

// Using anyhow just so I don't need to deal with error transformation but also not use unwrap.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Using 8888 instead of 8080 since that was what was given in example
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let mut readers = HashMap::new();
    let mut writers = HashMap::new();
    let mut ports_to_remove = Vec::new();
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
            readers.insert(port,reader.lines());
            writers.insert(port,writer);
        }
        // Then for each person check their threads.
        for (reader_port,reader) in readers.iter_mut(){
            if let Ok(Ok(maybe_line)) =  timeout(TIMEOUT_DURATION,reader.next_line()).await{
                // We rely on a graceful disconnection to remove the reader and writer.
                if let Some(line) = maybe_line{
                    for (writer_port,writer) in writers.iter_mut(){
                        if writer_port == reader_port{
                            writer.write(&format!("ACK:MESSAGE\n").into_bytes()).await.ok();
                            writer.flush().await.ok();
                        }else{
                           writer.write(&format!("Message:{} {}\n",reader_port,line).into_bytes()).await.ok();
                           writer.flush().await.ok();
                        }
                    }
                }else{
                    ports_to_remove.push(*reader_port);
                }
            }
        }
        //We then clean up any dead readers and writers.
        for port in ports_to_remove{
            readers.remove(&port);
            writers.remove(&port);
        }
        ports_to_remove = Vec::new();
    }
}