use std::{collections::HashMap, sync::Arc, time::SystemTime};
use message::{Message, Post, Acknowledgement};
use tokio::{net::{TcpListener, TcpStream}, sync::{Mutex, mpsc}, io::{AsyncReadExt, BufReader, AsyncBufReadExt, AsyncWriteExt}};

struct Topic {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<Post>>>>,
    data: Arc<Mutex<HashMap<String, Post>>>
}

impl Topic {
    pub fn new() -> Self {
        Self{
            subscribers: Arc::new(Mutex::new(Vec::new())),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
/// Used to create a Queute Server
#[derive(Default)]
struct Server {
    topics: Arc<Mutex<HashMap<String, Topic>>>
}

impl Server {
    pub fn new() -> Self {
        Server{
            topics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_connection(&self, mut stream: TcpStream) {

        let mut rdr = BufReader::new(&mut stream);
        let mut l = String::new();
        rdr.read_line(&mut l).await;

        match l.trim().split(' ').collect::<Vec<_>>().as_slice() {
            ["SUB", topic] => {
                // if user subscribes, add their subscription to the topic
                let tx;
                let mut rx;
                let tx_close;
                let mut rx_close;
                let topic = topic.to_string();
                let idx_of_subscriber;
                {
                    let topics = self.topics.lock().await;
                    match topics.get(&topic) {
                        Some(t) => {
                            // if the key is contained, set topic
                            (tx, rx) = mpsc::channel::<Post>(10);
                            (tx_close, rx_close) = mpsc::channel::<()>(1);
                            // add tx to subscription pool
                            let mut subs = t.subscribers.lock().await;
                            idx_of_subscriber = subs.len();
                            subs.push(tx);
                        },
                        None => {
                            // TODO: Add Queute configurations
                            // if in strict mode, fail
                            // if in dynamic mode, create topic
                            return
                        }
                    }
                }

                // handle user actions after subscription in separate thread
                tokio::spawn(async move {
                    loop {
                        let mut line = String::new();
                        if rdr.read_line(&mut line).await.unwrap() == 0 {
                            break;
                        }
                    }
                    tx_close.send(()).await;
                });

                loop {
                    tokio::select! {
                        Some(msg) = rx.recv() => {
                            stream.write_all(bson::to_bson(&msg).unwrap().to_string().as_bytes()).await;
                        }
                        Some(_) = rx_close.recv() => {
                            stream.write_all(b"Closed").await;
                        }
                    }
                }
            },
            ["PUB", topic, content @ ..] => {
                let topic = topic.to_string();
                // convert content to single string

            },
            ["ACK", topic, id] => {
                let topic = topic.to_string();
                let id = id.to_string();
                // client acknowledges posted message
                {
                    let topics = self.topics.lock().await;
                    match topics.get(&topic) {
                        Some(topic) => {
                            let data = topic.data.lock().await;
                            match data.get(&id) {
                                Some(d) => {
                                    // mark topic as acknowledged
                                    d.acknowledged = true;
                                },
                                None => {
                                    // invalid id
                                    stream.write_all(b"Invalid message specified").await;
                                }
                            }
                        },
                        None => {

                        }
                    }
                }
            },
            _ => {
                stream.write_all(b"Invalid connection specified").await;
            }
        }

        //let mut buf = [0; 1024];
        //let topic_msg;

        //// detect subscription or publish

        //// Read the topic selection from the client
        //match stream.read(&mut buf).await {
        //    Ok(n) => {
        //        let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
        //        println!("Received topic selection: {}", msg);
        //        topic_msg = msg;
        //    }
        //    Err(_) => {
        //        // TODO: write error message to stream
        //        return;
        //    }
        //}

        //let tx;
        //let rx;

        //// check for topic in server. If in strict topic mode, fail when
        ////  topic is not found
        //{
        //    let topics = self.topics.lock().await;
        //    match topics.get(&topic_msg) {
        //        Some(t) => {
        //            // if the key is contained, set topic
        //            (tx, rx) = mpsc::channel::<Message>(10);
        //            // add tx to subscription pool
        //        },
        //        None => {
        //            // TODO: Add Queute configurations
        //            // if in strict mode, fail
        //            // if in dynamic mode, create topic
        //            return
        //        }
        //    }
        //}

        //let (tx, _) = mpsc::channel::<Message>(10);
        //let idx_of_subscriber;

        //// add subscriber in scope to free mutex afterward
        //{
        //    let mut subscribers = self.subscribers.lock().await;
        //    idx_of_subscriber = subscribers.len();
        //    subscribers.push(tx.clone());
        //}
        //let mut expectingAck = false;

        //let mut buf = [0; 1024];
        //loop {
        //    match stream.read(&mut buf).await {
        //        Ok(0) => break, // connection closed and thread will be cleaned up
        //        Ok(n) => {
        //            // handle message
        //            let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
        //            println!("received message: {}", msg);

        //            // parse message into enum Message
        //            let message: Message = bson::from_bson(msg.into()).unwrap();

        //            // if post, broadcast Post to all subscribers
        //            match message {
        //                Message::Ack(ack) => {
        //                    if expectingAck {
        //                        continue;
        //                    } else {
        //                        // create error

        //                        // send back Message:Err serialized to BSON
        //                        todo!();
        //                    }
        //                },
        //                Message::Post(post) => {

        //                },
        //            }
        //        },
        //        Err(_) => {
        //            // error occured in connection
        //            break
        //        }
        //    }
        //}

        // remove subscriber in scope like before
        //{
        //    let mut subscribers = self.subscribers.lock().await;
        //    subscribers.remove(idx_of_subscriber);
        //}
    }

    async fn broadcast(&self, topic: &Topic, post: Post) {
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

    while let Ok((stream, _)) = listener.accept().await {
        let server_clone = server.clone();

        tokio::spawn(async move {
            server_clone.handle_connection(stream).await;
        });
    }
}
