use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use surreal_simple_client::SurrealClient;

#[derive(Serialize, Deserialize, Debug)]
struct User {
  id: String,
  username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = SurrealClient::new("ws://127.0.0.1:8000/rpc")
    .await
    .expect("RPC handshake error");

  client.signin("root", "root").await.expect("Signin error");
  client
    .use_namespace("modspot", "modspot")
    .await
    .expect("Namespace error");

  // clear the database in case a persistent database is used.
  client
    .send_query("delete User".to_owned(), Value::Null)
    .await
    .unwrap();

  create_user(&mut client, "John".to_owned()).await;

  let some_user: Option<User> = client
    .find_one("select * from User".to_owned(), Value::Null)
    .await
    .unwrap();

  let username = some_user.and_then(|u| Some(u.username));
  assert_eq!(Some("John".to_owned()), username);

  create_user(&mut client, "Mark".to_owned()).await;

  let users: Vec<User> = client
    .find_many("select * from User".to_owned(), Value::Null)
    .await
    .unwrap();

  let mut usernames: Vec<String> = users.into_iter().map(|u| u.username).collect();

  // sort the results to ensure they're always in the same order.
  usernames.sort();

  assert_eq!(vec!["John".to_owned(), "Mark".to_owned()], usernames);

  Ok(())
}

async fn create_user(client: &mut SurrealClient, username: String) {
  client
    .send_query(
      "create User set username = $username".to_owned(),
      json!({ "username": username }),
    )
    .await
    .unwrap();
}
