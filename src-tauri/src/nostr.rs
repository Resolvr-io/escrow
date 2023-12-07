use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use nostr_sdk::nips::nip04;
use nostr_sdk::prelude::{FromBech32, Keys, SecretKey};
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::{Client};
use nostr_sdk::{Filter, Kind, RelayMessage, RelayPoolNotification};
use tokio::sync::Mutex;
#[derive(Clone, Debug)]
pub struct NostrDirectMessage {
    msg: String,
    author: XOnlyPublicKey,
}

pub struct NostrNip04MessageProvider {
    keys: Keys,
    pub client: Client,
    pub queue: Arc<Mutex<VecDeque<NostrDirectMessage>>>,
}

pub struct QueueCursor<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    position: usize,
}

impl<T> QueueCursor<T>
where
    T: Clone,
{
    pub async fn next(&mut self) -> Option<T> {
        let queue = self.queue.lock().await;
        if let Some(item) = queue.get(self.position) {
            self.position += 1;
            return Some(item.clone());
        }

        None
    }

    pub async fn peek(&self) -> Option<T> {
        let queue = self.queue.lock().await;
        if let Some(item) = queue.get(self.position) {
            return Some(item.clone());
        }

        None
    }
}

impl NostrNip04MessageProvider {
    pub fn new(nostr_sk: String) -> Self {
        let secret_key = SecretKey::from_bech32(nostr_sk).unwrap();
        let keys = Keys::new(secret_key);
        let client = Client::new(&keys);

        Self {
            keys,
            client,
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.client.add_relay("wss://relay.damus.io", None).await?;
        self.client.add_relay("wss://nostr.wine", None).await?;

        let keys = self.keys.clone();

        let subscription = Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .pubkey(keys.public_key());

        self.client.connect().await;
        self.client.subscribe(vec![subscription]).await;

        self.stream_events()
    }

    fn stream_events(&mut self) -> Result<()> {
        let client = self.client.clone();
        let keys = self.keys.clone();

        let task_queue = self.queue.clone();
        tokio::spawn(async move {
            match client
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
                            match nip04::decrypt(&keys.secret_key()?, &event.pubkey, &event.content)
                            {
                                Ok(msg) => {
                                    let mut queue = task_queue.lock().await;
                                    queue.push_front(NostrDirectMessage {
                                        msg,
                                        author: event.pubkey,
                                    });
                                }
                                Err(e) => println!("Impossible to decrypt direct message: {e}"),
                            }
                        }
                    }
                    Ok(false) // Set to true to exit from the loop
                })
                .await
            {
                Ok(_) => {}
                Err(e) => println!("Failed to retrieve events: {:?}", e),
            };
        });

        Ok(())
    }

    pub fn get_queue_cursor(&self) -> QueueCursor<NostrDirectMessage> {
        QueueCursor {
            queue: self.queue.clone(),
            position: 0,
        }
    }
}
