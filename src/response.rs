use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// A raw, unparsed, response from the SurrealDB instance.
///
/// A surreal response always looks like the following:
/// ```json
/// // SELECT ->manage->Project as projects, * from Account;
/// [
///   { // <-- SurrealQueryResult: one object for each query passed in the request
///     "time": "229.2µs",
///     "status": "OK",
///     "result": [
///       { // <-- one object for each returned row
///         "projects": [...],
///         
///         // ... the rest of the fields from the Account node and returned by the
///         // * selector:
///         "id": "...",
///         "username": "..."
///       }
///
///     ]
///   },
/// ]
/// ```
///
/// The [`SurrealResponseData::result`] field holds a `serde::Value` that represents
/// such JSON. However it is recommended to use the [SurrealResponseData] methods
/// to make retrieving data easier.
#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealResponseData {
  pub id: String,

  pub result: SurrealResponseResult,
}

impl SurrealResponseData {
  /// Get the result of the `n`-th query out of the resulting JSON. Refer to the
  /// [SurrealResponseData] description to understand what the resulting JSON
  /// looks like and what object will be returned by this function.
  ///
  /// If the response doesn't contain any data then [None] is immediately returned
  pub fn get_nth_query_result(&self, n: usize) -> Option<&SurrealQueryResult> {
    match &self.result {
      SurrealResponseResult::Data(results) => results.get(n),
      _ => None,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SurrealResponseResult {
  String(String),
  Data(Vec<SurrealQueryResult>),
  Null,
}

/// A raw, unparsed response from the SurrealDB instance for a single statement.
///
/// A surreal response always looks like the following:
/// ```json
/// // SELECT ->manage->Project as projects, * from Account;
/// [
///   { // <-- SurrealQueryResult: one object for each query passed in the request
///     "time": "229.2µs",
///     "status": "OK",
///     "result": [
///       { // <-- one object for each returned row
///         "projects": [...],
///         
///         // ... the rest of the fields from the Account node and returned by the
///         // * selector:
///         "id": "...",
///         "username": "..."
///       }
///
///     ]
///   },
/// ]
/// ```
///
#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealQueryResult {
  pub time: String,
  pub status: String,

  result: Vec<Value>,
}

impl SurrealQueryResult {
  pub fn results(&self) -> &Vec<Value> {
    &self.result
  }

  /// Get the inner results and extract the
  /// [Value] out of the `key` for each row.
  ///
  /// The function filters out the rows where `key` returned [None]
  pub fn results_key(&self, key: &str) -> Vec<&Value> {
    self.results().iter().filter_map(|v| v.get(key)).collect()
  }
}
