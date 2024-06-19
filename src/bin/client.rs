use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },

    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val.clone()).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let tx2 = tx.clone();

    let get_req = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Get {
            key: "goo".to_string(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("got {:?}", res);
    });

    let set_req = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set {
            key: "goo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("got {:?}", res);
    });

    set_req.await.unwrap();
    get_req.await.unwrap();
    manager.await.unwrap();
}
