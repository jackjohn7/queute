use message::{Acknowledgement, Message, Post};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    net::SocketAddr,
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

struct Topic {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<Post>>>>,
    data: Arc<Mutex<HashMap<String, Post>>>,
    acks: Arc<Mutex<HashSet<String>>>,
}

impl Topic {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            data: Arc::new(Mutex::new(HashMap::new())),
            acks: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
/// Used to create a Queute Server
#[derive(Default)]
struct Server {
    topics: Arc<Mutex<HashMap<String, Topic>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            topics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_connection(&self, mut stream: TcpStream, _addr: SocketAddr) {
        //let mut rdr = BufReader::new(&mut stream);
        //let mut l = String::new();
        let mut buf = vec![0; 1024];
        let _ = stream.read(&mut buf);
        //let _ = rdr.read_line(&mut l).await;
        // stringify the bytes
        let msg;
        match String::from_utf8(buf) {
            Ok(msg_content) => msg = msg_content,
            Err(_) => {
                let _ = stream.write_all("Connection failed".as_bytes()).await;
                return;
            }
        }

        // TODO: parse initial message and expect credentials

        match msg.trim().split(' ').collect::<Vec<_>>().as_slice() {
            ["SUB", topic] => {
                // channel for sending subscription information
                let topic = topic.to_string();
                let (t_post, mut r_post);
                // might have to switch to something key-value since unsubscribing
                //  will invalidate this index
                let _idx_of_subscriber;
                {
                    let topics = self.topics.lock().await;
                    match topics.get(&topic) {
                        Some(t) => {
                            // if the key is contained, set topic
                            (t_post, r_post) = mpsc::channel::<Post>(10);
                            // add tx to subscription pool
                            let mut subs = t.subscribers.lock().await;
                            _idx_of_subscriber = subs.len();
                            subs.push(t_post);
                        }
                        None => {
                            // TODO: Add Queute configurations
                            // if in strict mode, fail
                            // if in dynamic mode, create topic
                            return;
                        }
                    }
                }
                // clear buf
                buf = vec![0; 1024];

                loop {
                    tokio::select! {
                        _ = stream.read(&mut buf) => {
                            // you now have read into i
                        },
                        val = r_post.recv() => {
                            // you can now write to stream with new post
                            match val {
                                Some(post) => {
                                    let _ = stream.write_all(bson::to_bson(&post).unwrap().to_string().as_bytes());
                                },
                                None => break
                            }
                        }
                    }
                }
            }
            _ => {
                // invalid message
                let _ = stream.write_all("Connection failed".as_bytes()).await;
                return;
            }
        }
    }

    async fn _broadcast(&self, topic: &Topic, post: Post) {
        let subscribers = topic.subscribers.lock().await;
        for subscriber in subscribers.iter() {
            // cloning here could be bad. Should look into using Arc
            let _ = subscriber.send(post.clone()).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let server = Arc::new(Server::new());
    let addr = "127.0.0.1:8080";

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    println!("Queute running on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let server_clone = server.clone();

        tokio::spawn(async move {
            server_clone.handle_connection(stream, addr).await;
        });
    }
}
