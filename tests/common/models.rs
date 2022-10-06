use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use surreal_simple_client::rpc::RpcResult;
use surreal_simple_client::SurrealClient;
use surreal_simple_querybuilder::prelude::Foreign;
use surreal_simple_querybuilder::prelude::ToNodeBuilder;
use surreal_simple_querybuilder::querybuilder::QueryBuilder;
use surreal_simple_querybuilder::querybuilder::QueryBuilderSetObject;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
  #[serde(skip_serializing)]
  pub id: Option<String>,
  pub name: String,
}

pub mod user_schema {
  use surreal_simple_querybuilder::model;

  model!(User { id, name });
}

use user_schema::schema::model as user;

impl User {
  pub fn new(name: String) -> Self {
    Self { id: None, name }
  }

  pub fn into_json(&self) -> Value {
    serde_json::to_value(self).unwrap()
  }

  pub async fn create(&self, client: &mut SurrealClient) -> RpcResult<()> {
    client
      .send_query(
        QueryBuilder::new()
          .create(user)
          .set_object::<Self>()
          .build(),
        self.into_json(),
      )
      .await?
      .await?;

    Ok(())
  }

  pub async fn find_by_name(client: &mut SurrealClient, name: &str) -> RpcResult<Option<Self>> {
    let result: Option<Self> = client
      .find_one(
        QueryBuilder::new()
          .select(user)
          .filter(user.name.equals_parameterized())
          .build(),
        json!({ "name": name }),
      )
      .await?;

    println!("It never reaches this");

    Ok(result)
  }
}

impl QueryBuilderSetObject for User {
  fn set_querybuilder_object<'a>(mut querybuilder: QueryBuilder<'a>) -> QueryBuilder {
    let a = &[
      querybuilder.hold(user.id.equals_parameterized()),
      querybuilder.hold(user.name.equals_parameterized()),
    ];

    querybuilder.set_many(a)
  }
}

// -----------------------------------------------------------------------------

pub struct File {
  pub id: Option<String>,
  pub name: String,
  pub author: Foreign<User>,
}

pub mod file_schema {
  use super::user_schema::schema::User;
  use surreal_simple_querybuilder::model;

  model!(File {
    id,
    name,
    author<User>
  });
}
