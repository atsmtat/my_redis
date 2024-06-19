use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

fn compute_hash<K>(key: &K) -> u64
where
    K: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
struct ShardedDb(Arc<Vec<Mutex<HashMap<String, Bytes>>>>);

impl ShardedDb {
    fn new(num_shards: usize) -> Self {
        let mut vec = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            vec.push(Mutex::new(HashMap::new()));
        }
        Self(Arc::new(vec))
    }

    fn get(&self, key: &str) -> Option<Bytes> {
        let index = compute_hash(key) as usize % self.0.len();
        let shard = self.0[index].lock().unwrap();
        shard.get(key).cloned()
    }

    fn insert(&self, key: String, val: Bytes) {
        let index = compute_hash(&key) as usize % self.0.len();
        let mut shard = self.0[index].lock().unwrap();
        shard.insert(key, val);
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = ShardedDb::new(4);
    println!("Listening...");

    loop {
        let (stream, remote) = listener.accept().await.unwrap();
        println!("Accepted new connection from {:?}", remote);

        let db = db.clone();
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(val) = db.get(cmd.key()) {
                    Frame::Bulk(val)
                } else {
                    Frame::Null
                }
            }
            _ => {
                panic!("unimplemented!");
            }
        };
        connection.write_frame(&response).await.unwrap();
    }
}
