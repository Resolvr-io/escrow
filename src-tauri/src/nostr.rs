use anyhow::Result;
use dlc_messages::{AcceptDlc, Message, OfferDlc, SignDlc};
use nostr_sdk::nips::nip04;
use nostr_sdk::prelude::{Keys, SecretKey};
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::Client;
use nostr_sdk::{Filter, Kind, RelayMessage, RelayPoolNotification};
use secp256k1_zkp::Parity;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolvrEscrowMessage {
    OfferDlc((OfferDlc, Parity)),
    AcceptDlc((AcceptDlc, Parity)),
    SignDlc((SignDlc, Parity)),
}

impl ResolvrEscrowMessage {
    fn to_encoded_hex_string(&self) -> Result<String> {
        Ok(hex::encode(bincode::serialize(self)?))
    }

    fn from_encoded_hex_string(encoded_hex_string: &str) -> Result<Self> {
        Ok(bincode::deserialize(&hex::decode(encoded_hex_string)?)?)
    }

    pub fn to_dlc_message(self) -> (Message, Parity) {
        match self {
            ResolvrEscrowMessage::OfferDlc((offer_dlc, pubkey_parity)) => {
                (Message::Offer(offer_dlc), pubkey_parity)
            }
            ResolvrEscrowMessage::AcceptDlc((accept_dlc, pubkey_parity)) => {
                (Message::Accept(accept_dlc), pubkey_parity)
            }
            ResolvrEscrowMessage::SignDlc((sign_dlc, pubkey_parity)) => {
                (Message::Sign(sign_dlc), pubkey_parity)
            }
        }
    }

    pub fn from_dlc_message(msg: Message, pubkey_parity: Parity) -> Option<Self> {
        match msg {
            Message::Offer(offer_dlc) => {
                Some(ResolvrEscrowMessage::OfferDlc((offer_dlc, pubkey_parity)))
            }
            Message::Accept(accept_dlc) => {
                Some(ResolvrEscrowMessage::AcceptDlc((accept_dlc, pubkey_parity)))
            }
            Message::Sign(sign_dlc) => {
                Some(ResolvrEscrowMessage::SignDlc((sign_dlc, pubkey_parity)))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NostrEncodedDirectMessage {
    pub sender: nostr_sdk::prelude::XOnlyPublicKey,
    pub msg: ResolvrEscrowMessage,
}

pub struct NostrNip04MessageProvider {
    client: Client,
    rx: Arc<Mutex<Receiver<NostrEncodedDirectMessage>>>,
    notifications_handle: tokio::task::JoinHandle<()>,
}

impl NostrNip04MessageProvider {
    pub async fn new(secret_key: SecretKey) -> Result<Self> {
        let keys = Keys::new(secret_key);
        let client = Client::new(&keys);

        // TODO: Add relays from an input argument.
        client.add_relay("wss://relay.damus.io", None).await?;
        client.add_relay("wss://nostr.wine", None).await?;

        client.connect().await;

        client
            .subscribe(vec![Filter::new()
                .kind(Kind::EncryptedDirectMessage)
                .pubkey(keys.public_key())])
            .await;

        let (tx, rx): (
            Sender<NostrEncodedDirectMessage>,
            Receiver<NostrEncodedDirectMessage>,
        ) = std::sync::mpsc::channel();

        let notifications_task_client = client.clone();
        let notifications_handle = tokio::task::spawn(async move {
            match notifications_task_client
                .handle_notifications(|notification| async {
                    if let RelayPoolNotification::Message(
                        _url,
                        RelayMessage::Event {
                            subscription_id: _,
                            event,
                        },
                    ) = notification
                    {
                        if event.kind == Kind::EncryptedDirectMessage {
                            if let Ok(msg) =
                                nip04::decrypt(&secret_key, &event.pubkey, &event.content)
                            {
                                if let Ok(msg) = ResolvrEscrowMessage::from_encoded_hex_string(&msg)
                                {
                                    let _ = tx.send(NostrEncodedDirectMessage {
                                        sender: event.pubkey,
                                        msg,
                                    });
                                }
                            }
                        }
                    }
                    // Continue handling notifications until the end of time.
                    // The task is eventually aborted by the destructor (see the Drop trait implementation below).
                    Ok(false)
                })
                .await
            {
                Ok(_) => {}
                Err(e) => println!("Failed to retrieve events: {:?}", e),
            };
        });

        Ok(Self {
            client,
            rx: Arc::from(Mutex::from(rx)),
            notifications_handle,
        })
    }

    pub fn pop(&self) -> Option<NostrEncodedDirectMessage> {
        self.rx.lock().unwrap().try_recv().ok()
    }

    pub async fn send(&self, recipient: XOnlyPublicKey, msg: ResolvrEscrowMessage) -> Result<()> {
        self.client
            .send_direct_msg(
                recipient,
                ResolvrEscrowMessage::to_encoded_hex_string(&msg)?,
                None,
            )
            .await?;
        Ok(())
    }
}

impl Drop for NostrNip04MessageProvider {
    fn drop(&mut self) {
        self.notifications_handle.abort();
    }
}
