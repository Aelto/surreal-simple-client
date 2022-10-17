use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use surreal_simple_client::rpc::RpcResult;
use surreal_simple_client::SurrealClient;
use surreal_simple_querybuilder::prelude::ForeignVec;
use surreal_simple_querybuilder::prelude::IntoKey;
use surreal_simple_querybuilder::prelude::NodeBuilder;
use surreal_simple_querybuilder::prelude::ToNodeBuilder;
use surreal_simple_querybuilder::querybuilder::QueryBuilder;
use surreal_simple_querybuilder::querybuilder::QueryBuilderSetObject;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  #[serde(skip_serializing)]
  pub id: Option<String>,
  pub name: String,

  #[serde(default)]
  pub written_files: ForeignVec<File>,
}

pub mod user_schema {
  use super::file_schema::schema::File;
  use surreal_simple_querybuilder::model;

  model!(User {
   id,
   name,
   ->write->File as written_files
  });
}

use user_schema::schema::model as user;

impl IntoKey<String> for User {
  fn into_key<E>(&self) -> Result<String, E>
  where
    E: serde::ser::Error,
  {
    self
      .id
      .as_ref()
      .map(String::clone)
      .ok_or(serde::ser::Error::custom("The user has no ID"))
  }
}

impl User {
  pub fn new(name: String) -> Self {
    Self {
      id: None,
      name,
      written_files: ForeignVec::Unloaded,
    }
  }

  pub fn into_json(&self) -> Value {
    serde_json::to_value(self).unwrap()
  }

  pub async fn create(&self, socket: &mut SurrealClient) -> RpcResult<Option<Self>> {
    socket
      .find_one(
        QueryBuilder::new()
          .create(user)
          .set_object::<Self>()
          .build(),
        self.into_json(),
      )
      .await
  }

  pub async fn relate_with_file(
    client: &mut SurrealClient, user_id: &str, file_id: &str,
  ) -> RpcResult<()> {
    client
      .send_query(
        QueryBuilder::new()
          .relate(user_id.with(user.written_files.name()).with(&file_id))
          .build(),
        json!({}),
      )
      .await?;

    Ok(())
  }

  pub async fn find_by_name(client: &mut SurrealClient, name: &str) -> RpcResult<Option<Self>> {
    client
      .find_one(
        QueryBuilder::new()
          .select("*")
          .from(user)
          .filter(user.name.equals_parameterized())
          .build(),
        json!({ "name": name }),
      )
      .await
  }
}

impl QueryBuilderSetObject for User {
  fn set_querybuilder_object<'a>(mut querybuilder: QueryBuilder<'a>) -> QueryBuilder {
    let a = &[querybuilder.hold(user.name.equals_parameterized())];

    querybuilder.set_many(a)
  }
}

// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
  #[serde(skip_serializing)]
  pub id: Option<String>,
  pub name: String,

  #[serde(default)]
  pub authors: ForeignVec<User>,
}

pub mod file_schema {
  use super::user_schema::schema::User;
  use surreal_simple_querybuilder::model;

  model!(File {
    id,
    name,

    <-write<-User as authors
  });
}

impl IntoKey<String> for File {
  fn into_key<E>(&self) -> Result<String, E>
  where
    E: serde::ser::Error,
  {
    self
      .id
      .as_ref()
      .map(String::clone)
      .ok_or(serde::ser::Error::custom("The file has no ID"))
  }
}

impl File {
  pub fn into_json(&self) -> Value {
    serde_json::to_value(self).unwrap()
  }

  pub fn new(name: String) -> Self {
    Self {
      name,
      id: None,
      authors: ForeignVec::Unloaded,
    }
  }

  pub async fn create(&self, socket: &mut SurrealClient) -> RpcResult<Option<Self>> {
    socket
      .find_one(
        QueryBuilder::new()
          .create(user)
          .set_object::<Self>()
          .build(),
        self.into_json(),
      )
      .await
  }
}

impl QueryBuilderSetObject for File {
  fn set_querybuilder_object<'a>(mut querybuilder: QueryBuilder<'a>) -> QueryBuilder {
    let a = &[querybuilder.hold(user.name.equals_parameterized())];

    querybuilder.set_many(a)
  }
}
