use std::error::Error;
use std::{io, net::SocketAddr};

use did::{print_qr_code, DidDocument, VerificationMethod, DID};
use futures::stream::StreamExt;
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
    select,
    sync::{
        mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
    try_join,
};
use tokio_util::codec::FramedRead;

static CONTEXT: &str = "Client";

use crate::ClientId;
use crate::{
    main_loop::{ServerHandle, ToDelivery},
    telnet::{Item, TelnetCodec},
};

/// Messages received from the main loop.
pub enum FromDelivery {
    // Should be decrypted data
    Message(Vec<u8>),
    QR(String),
}

#[derive(Debug, Clone)]
pub enum ClientRole {
    Holder,
    Issuer,
    Verifier,
}
#[derive(Debug)]
pub struct InvalidClientRoleError;

impl std::fmt::Display for InvalidClientRoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid client role")
    }
}

impl Error for InvalidClientRoleError {}

impl TryFrom<String> for ClientRole {
    type Error = InvalidClientRoleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "holder" => Ok(ClientRole::Holder),
            "issuer" => Ok(ClientRole::Issuer),
            "verifier" => Ok(ClientRole::Verifier),
            _ => Err(InvalidClientRoleError),
        }
    }
}

/// This struct is constructed by the accept loop and used as the argument to
/// `spawn_client`.
pub struct ClientInfo {
    pub id: ClientId,
    pub ip: SocketAddr,
    pub handle: ServerHandle,
    pub tcp: TcpStream,
}

struct ClientData {
    id: ClientId,
    handle: ServerHandle,
    recv: Receiver<FromDelivery>,
    tcp: TcpStream,
}

/// A handle to this actor, used by the server.
#[derive(Debug)]
pub struct ClientHandle {
    pub id: ClientId,
    ip: SocketAddr,
    chan: Sender<FromDelivery>,
    kill: JoinHandle<()>,
    pub role: Option<ClientRole>,
}

impl ClientHandle {
    pub fn send(&mut self, msg: FromDelivery) -> Result<(), io::Error> {
        if self.chan.try_send(msg).is_err() {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Can't keep up or dead",
            ))
        } else {
            Ok(())
        }
    }
    /// Kill the actor.
    pub fn kill(self) {
        // run the destructor
        drop(self);
    }
}

impl Drop for ClientHandle {
    fn drop(&mut self) {
        self.kill.abort()
    }
}

pub fn spawn_client(info: ClientInfo) {
    let (send, recv) = channel(64);

    let data = ClientData {
        id: info.id,
        handle: info.handle.clone(),
        tcp: info.tcp,
        recv,
    };

    // This spawns the new task.
    let (my_send, my_recv) = oneshot::channel();
    let kill = tokio::spawn(start_client(my_recv, data));

    // Then we create a ClientHandle to this new task, and use the oneshot
    // channel to send it to the task.
    let handle = ClientHandle {
        id: info.id,
        ip: info.ip,
        chan: send,
        kill,
        role: None,
    };

    // Ignore send errors here. Should only happen if the server is shutting
    // down.
    let _ = my_send.send(handle);
}

async fn start_client(my_handle: oneshot::Receiver<ClientHandle>, mut data: ClientData) {
    // Wait for `spawn_client` to send us the `ClientHandle` so we can forward
    // it to the main loop. We need the oneshot channel because we cannot
    // otherwise get the `JoinHandle` returned by `tokio::spawn`. We forward it
    // from here instead of in `spawn_client` because we want the server to see
    // the NewClient message before this actor starts sending other messages.
    let my_handle = match my_handle.await {
        Ok(my_handle) => my_handle,
        Err(_) => return,
    };
    data.handle.send(ToDelivery::NewClient(my_handle)).await;

    // We sent the client handle to the main loop. Start talking to the tcp
    // connection.
    let res = client_loop(data).await;
    match res {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Something went wrong: {}.", err);
        }
    }
}

/// This method performs the actual job of running the client actor.
async fn client_loop(mut data: ClientData) -> Result<(), io::Error> {
    let (read, write) = data.tcp.split();

    // communication between tcp_read and tcp_write
    let (send, recv) = unbounded_channel();

    let ((), ()) = try_join! {
        tcp_read(data.id, read, data.handle, send),
        tcp_write(write, data.recv, recv),
    }?;

    let _ = data.tcp.shutdown().await;

    Ok(())
}

#[derive(Debug)]
enum InternalMsg {
    GotAreYouThere,
    SendDont(u8),
    SendWont(u8),
    SendDo(u8),
}

async fn tcp_read(
    id: ClientId,
    read: ReadHalf<'_>,
    mut handle: ServerHandle,
    to_tcp_write: UnboundedSender<InternalMsg>,
) -> Result<(), io::Error> {
    let mut telnet = FramedRead::new(read, TelnetCodec::new());

    while let Some(item) = telnet.next().await {
        match item? {
            Item::AreYouThere => {
                to_tcp_write
                    .send(InternalMsg::GotAreYouThere)
                    .expect("Should not be closed.");
            }
            Item::GoAhead => { /* ignore */ }
            Item::InterruptProcess => return Ok(()),
            Item::Will(3) => {
                // suppress go-ahead
                to_tcp_write
                    .send(InternalMsg::SendDo(3))
                    .expect("Should not be closed.");
            }
            Item::Will(i) => {
                to_tcp_write
                    .send(InternalMsg::SendDont(i))
                    .expect("Should not be closed.");
            }
            Item::Do(i) => {
                to_tcp_write
                    .send(InternalMsg::SendWont(i))
                    .expect("Should not be closed.");
            }
            Item::Line(line) => {
                handle.send(ToDelivery::Message(id, line)).await;
            }
            Item::CreateDID => {
                let did = DID::generate();

                println!("[{}] creating did: {}", CONTEXT, did.id);
                let mut did_doc = DidDocument::new(&did.id);
                let ver_method_id_1 = format!("{}#key1", did);
                let verification_method = VerificationMethod {
                    id: ver_method_id_1.to_string(),
                    vc_type: "Ed25519VerificationKey2020".to_string(),
                    controller: did.to_string(),
                    public_key_hex: None,
                    public_key_base58: Some("SigningKey".into()),
                };
                did_doc.add_verification_method(verification_method);

                // Add authentication
                did_doc.add_authentication(&ver_method_id_1);
                println!("[{}] creating did document", CONTEXT);
                handle.send(ToDelivery::DidDocument(id, did_doc)).await;
            }
            Item::ShowDID(did) => {
                let readalbe_string = String::from_utf8(did.clone()).expect("Failed to parsed");
                println!("[{}] show did: {}", CONTEXT, readalbe_string);
                handle.send(ToDelivery::ShowDocument(id, did)).await;
            }
            Item::AssignRole(role) => {
                let role = String::from_utf8(role.clone()).expect("Failed to parsed");
                println!("[{}] Assinging new role: {}", CONTEXT, role);
                handle
                    .send(ToDelivery::NewRole(
                        id,
                        role.try_into().expect("Failed to parse role"),
                    ))
                    .await;
            }
            Item::WhoAmI => {
                println!("[{}] Asking for who they are", CONTEXT);
                handle.send(ToDelivery::MyInfo(id)).await;
            }
            Item::VerifyDID(did) => {
                let readalbe_string = String::from_utf8(did.clone()).expect("Failed to parsed");
                println!("[{}] Verifying did: {}", CONTEXT, readalbe_string);
                handle.send(ToDelivery::VerifyDID(id, did)).await;
            }
            Item::ShowVP => {
                println!("[{}] Verifying Presentation", CONTEXT);
                handle.send(ToDelivery::ShowVP(id)).await;
            }
            //Todo: Add command direction to server
            item => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unable to handle {:?}", item),
                ));
            }
        }
    }

    // disconnected

    Ok(())
}

async fn tcp_write(
    mut write: WriteHalf<'_>,
    mut recv: Receiver<FromDelivery>,
    mut from_tcp_read: UnboundedReceiver<InternalMsg>,
) -> Result<(), io::Error> {
    loop {
        select! {
            msg = recv.recv() => match msg {
                Some(FromDelivery::Message(msg)) => {
                    write.write_all(&msg).await?;
                    write.write_all(&[13, 10]).await?;
                },
                Some(FromDelivery::QR(url)) => {
                    let qr = print_qr_code(&url).unwrap();
                    println!("[{}] Receving QR which encoded url: {}", CONTEXT, url);
                    write.write_all(&qr.into_bytes()).await?;
                    write.write_all(&[13, 10]).await?;
                },
                None => {
                    break;
                },
            },
            msg = from_tcp_read.recv() => match msg {
                Some(InternalMsg::GotAreYouThere) => {
                    write.write_all(b"Yes.\r\n").await?;
                },
                Some(InternalMsg::SendDont(i)) => {
                    write.write_all(&[0xff, 254, i]).await?;
                },
                Some(InternalMsg::SendWont(i)) => {
                    write.write_all(&[0xff, 252, i]).await?;
                },
                Some(InternalMsg::SendDo(i)) => {
                    write.write_all(&[0xff, 253, i]).await?;
                },
                None => {
                    break;
                },
            },
        };
    }

    Ok(())
}
