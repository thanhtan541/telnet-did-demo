use did::{DidDocument, DidStorage};
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::{
    client::{ClientHandle, ClientRole, FromDelivery},
    ClientId,
};

// Define the messages the actor can handle
pub enum ToDelivery {
    NewClient(ClientHandle),
    NewRole(ClientId, ClientRole),
    MyInfo(ClientId),
    Message(ClientId, Vec<u8>),
    ShowDocument(ClientId, Vec<u8>),
    DidDocument(ClientId, DidDocument),
    FatalError(io::Error),
}

/// This struct is used by client actors to send messages to the main loop. The
/// message type is `ToDelivery`.
#[derive(Clone, Debug)]
pub struct ServerHandle {
    chan: Sender<ToDelivery>,
    next_id: Arc<AtomicUsize>,
}

impl ServerHandle {
    pub async fn send(&mut self, msg: ToDelivery) {
        if self.chan.send(msg).await.is_err() {
            panic!("Main loop has shut down.");
        }
    }

    pub fn next_id(&self) -> ClientId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        ClientId(id)
    }
}

#[derive(Default, Debug)]
struct Data {
    clients: HashMap<ClientId, ClientHandle>,
}

pub fn spawn_main_loop() -> (ServerHandle, JoinHandle<()>) {
    let (send, recv) = channel(64);

    let handle = ServerHandle {
        chan: send,
        next_id: Default::default(),
    };

    let join = tokio::spawn(async move {
        let res = main_loop(recv).await;
        match res {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Oops {}.", err);
            }
        }
    });

    (handle, join)
}

async fn main_loop(mut recv: Receiver<ToDelivery>) -> Result<(), io::Error> {
    let mut data = Data::default();
    let mut did_storage = DidStorage::new();

    while let Some(msg) = recv.recv().await {
        match msg {
            ToDelivery::NewClient(handle) => {
                println!("[Delivery Service] received new client");
                data.clients.insert(handle.id, handle);

                let msg_to_client = "Welcome!";
                let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == handle.id {
                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };

                        break;
                    }
                }
            }
            ToDelivery::Message(from_id, msg) => {
                // If we fail to send messages to any actor, we need to remove
                // it, but we can't do so while iterating.
                // let mut to_remove = Vec::new();

                println!("[Delivery Service] received message");
                // Iterate through clients so we can send the message.
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        continue;
                    }

                    let msg = FromDelivery::Message(msg.clone());

                    match handle.send(msg) {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("[Delivery Service] Something went wrong: {}.", err);
                        }
                    };
                }
            }
            ToDelivery::DidDocument(from_id, document) => {
                println!(
                    "[Delivery Service] insert document with id: {}",
                    document.id
                );
                let doc_id = document.id.clone();
                match did_storage.store(doc_id, document) {
                    Ok(_) => println!("[Delivery Service] Insert successfully"),
                    Err(_) => println!("[Delivery Service] Failed to insert"),
                }
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        let msg_to_client = "Your Did Document is saved!";
                        let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            ToDelivery::ShowDocument(from_id, did) => {
                let did = String::from_utf8(did).expect("Failed to parsed");
                println!("[Delivery Service] look up document with id: {}", did);
                let msg_to_client = match did_storage.get(&did) {
                    Some(doc) => doc.to_json().expect("Failed to parsed"),
                    None => "Not found".into(),
                };
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            ToDelivery::NewRole(from_id, role) => {
                println!("[Delivery Service] Updating role: {:?}", role.clone());
                let msg_to_client = format!("Hello {:?}", role.clone());
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        handle.role = Some(role.clone());
                        let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            ToDelivery::MyInfo(from_id) => {
                println!("[Delivery Service] Responding to who you are");
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        let role = match &handle.role {
                            Some(r) => format!("{:?}", r),
                            None => "Anonymous".into(),
                        };
                        let msg_to_client = format!("Hello {:?}", role);
                        let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            //Todo: add server logic
            ToDelivery::FatalError(err) => return Err(err),
        }
    }

    Ok(())
}
