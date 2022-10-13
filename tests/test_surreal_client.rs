#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod common;

use std::sync::Arc;
use std::sync::Mutex;

use common::models::File;
use common::models::User;
use common::open_connection;
use surreal_simple_client::rpc::RpcResult;
use surreal_simple_client::SurrealClient;

const USER0_NAME: &'static str = "User0";

#[tokio::test]
async fn it_connects() {
  let client = open_connection().await;

  assert!(
    client.is_ok(),
    "Client failed to connect to local testing database"
  );
}

#[tokio::test]
async fn it_connects_simultaneously() {
  let client_one = open_connection().await;
  let client_two = open_connection().await;

  assert!(
    client_one.is_ok() && client_two.is_ok(),
    "Failed to open two Surreal clients at once"
  );
}

/// A rather unintesting test, it is there to confirm a [SurrealClient] can be
/// passed between threads using an arc mutex. Which is useful for
/// multi-threaded environments like web frameworks.
#[tokio::test]
async fn it_supports_send() -> RpcResult<()> {
  let client = open_connection().await?;
  let shared_client = Arc::new(Mutex::new(client));

  assert!(!shared_client.is_poisoned());

  Ok(())
}

async fn it_creates_data(client: &mut SurrealClient) -> RpcResult<()> {
  let new_user = User::new(USER0_NAME.to_owned());
  if let Some(created_user) = new_user.create(client).await? {
    let new_file = File::new("LoremIpsum".to_owned());

    if let Some(created_file) = new_file.create(client).await? {
      User::relate_with_file(client, &created_user.id.unwrap(), &created_file.id.unwrap()).await?;
    }
  }

  Ok(())
}

#[tokio::test]
async fn it_fetches_data() -> RpcResult<()> {
  let mut client = open_connection().await?;

  it_creates_data(&mut client).await?;

  let fetch_result = User::find_by_name(&mut client, USER0_NAME).await;

  assert!(
    fetch_result.is_ok(),
    "Failed to fetch the user from the database"
  );

  let some_user_name = fetch_result?.map(|user| user.name);
  let expected_result = Some(USER0_NAME.to_owned());

  assert_eq!(expected_result, some_user_name);

  Ok(())
}
