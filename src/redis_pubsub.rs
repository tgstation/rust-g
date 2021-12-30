use redis::{Client, Commands, RedisError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

static ERROR_CHANNEL: &str = "RUSTG_REDIS_ERROR_CHANNEL";

thread_local! {
    static REQUEST_SENDER: RefCell<Option<flume::Sender<PubSubRequest>>> = RefCell::new(None);
    static RESPONSE_RECEIVER: RefCell<Option<flume::Receiver<PubSubResponse>>> = RefCell::new(None);
}

enum PubSubRequest {
    Subscribe(String),
    Publish(String, String),
}

// response might not be a good name, since those are not sent in response to requests
enum PubSubResponse {
    Disconnected(String),
    Message(String, String),
}

fn handle_redis_inner(
    client: Client,
    control: &flume::Receiver<PubSubRequest>,
    out: &flume::Sender<PubSubResponse>,
) -> Result<(), RedisError> {
    let mut conn = client.get_connection()?;
    let mut pub_conn = client.get_connection()?;
    let mut pubsub = conn.as_pubsub();
    pubsub.set_read_timeout(Some(Duration::from_secs(1)))?;

    loop {
        loop {
            match control.try_recv() {
                Ok(req) => match req {
                    PubSubRequest::Subscribe(channel) => {
                        pubsub.subscribe(&[channel.as_str()])?;
                    }
                    PubSubRequest::Publish(channel, message) => {
                        pub_conn.publish(&channel, &message)?;
                    }
                },
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => return Ok(()),
            }
        }

        if let Some(msg) = match pubsub.get_message() {
            Ok(msg) => Some(msg),
            Err(e) => {
                if e.is_timeout() {
                    None
                } else {
                    return Err(e);
                }
            }
        } {
            let chan = msg.get_channel_name().to_owned();
            let data: String = msg.get_payload().unwrap_or_default();
            if let Err(flume::TrySendError::Disconnected(_)) =
                out.try_send(PubSubResponse::Message(chan, data))
            {
                return Ok(()); // If no one wants to receive any more messages from us, we exit this thread
            }
        }
    }
}

fn handle_redis(
    client: Client,
    control: flume::Receiver<PubSubRequest>,
    out: flume::Sender<PubSubResponse>,
) {
    if let Err(e) = handle_redis_inner(client, &control, &out) {
        let _ = out.send(PubSubResponse::Disconnected(e.to_string()));
    }
}

fn connect(addr: &str) -> Result<(), RedisError> {
    let client = redis::Client::open(addr)?;
    let _ = client.get_connection_with_timeout(Duration::from_secs(1))?;
    let (c_sender, c_receiver) = flume::bounded(1000);
    let (o_sender, o_receiver) = flume::bounded(1000);
    REQUEST_SENDER.with(|cell| cell.replace(Some(c_sender)));
    RESPONSE_RECEIVER.with(|cell| cell.replace(Some(o_receiver)));
    thread::spawn(|| handle_redis(client, c_receiver, o_sender));
    Ok(())
}

fn disconnect() {
    // Dropping the sender and receiver will cause the other thread to exit
    REQUEST_SENDER.with(|cell| {
        cell.replace(None);
    });
    RESPONSE_RECEIVER.with(|cell| {
        cell.replace(None);
    });
}

// It's lame as hell to use strings as errors, but I don't feel like
// making a whole new type encompassing possible errors, since we're returning a string
// to BYOND anyway.
fn subscribe(channel: &str) -> Option<String> {
    return REQUEST_SENDER.with(|cell| {
        if let Some(chan) = cell.borrow_mut().as_ref() {
            return chan
                .try_send(PubSubRequest::Subscribe(channel.to_owned()))
                .err()
                .map(|e| e.to_string());
        };
        Some("Not connected".to_owned())
    });
}

fn publish(channel: &str, msg: &str) -> Option<String> {
    return REQUEST_SENDER.with(|cell| {
        if let Some(chan) = cell.borrow_mut().as_ref() {
            return chan
                .try_send(PubSubRequest::Publish(channel.to_owned(), msg.to_owned()))
                .err()
                .map(|e| e.to_string());
        };
        Some("Not connected".to_owned())
    });
}

fn get_messages() -> String {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    RESPONSE_RECEIVER.with(|cell| {
        let opt = cell.borrow_mut();
        if let Some(recv) = opt.as_ref() {
            for resp in recv.try_iter() {
                match resp {
                    PubSubResponse::Message(chan, msg) => {
                        result.entry(chan).or_default().push(msg);
                    }
                    PubSubResponse::Disconnected(error) => {
                        // Pardon the in-band signaling but it's probably the best way to do this
                        result
                            .entry(ERROR_CHANNEL.to_owned())
                            .or_default()
                            .push(error);
                    }
                }
            }
        }
    });

    serde_json::to_string(&result).unwrap_or("{}".to_owned())
}

byond_fn! { redis_connect(addr) {
    connect(addr).err().map(|e| e.to_string())
} }

byond_fn! { redis_disconnect() {
    disconnect();
    Some("")
} }

byond_fn! { redis_subscribe(channel) {
    subscribe(channel)
} }

byond_fn! { redis_get_messages() {
    Some(get_messages())
} }

byond_fn! { redis_publish(channel, message) {
    publish(channel, message)
} }
