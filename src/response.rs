use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// A raw, unparsed, response from the SurrealDB instance.
#[derive(Serialize, Deserialize, Debug)]
pub struct SurrealResponseData {
  pub id: String,
  pub result: Value,
}

impl SurrealResponseData {
  /// Find the **first** result with the given alias from the response. If [None] is passed
  /// as the alias then it defaults to `result`, which is the default alias for
  /// surrealdb results.
  ///
  /// This is the equivalent of calling [`Self.get_nth_aliased_results(0, alias)`]
  pub fn get_aliased_results(&self, alias: Option<&str>) -> Option<&Vec<Value>> {
    self.get_nth_aliased_results(0, alias)
  }

  /// Find the result with the given alias from the response. If [None] is passed
  /// as the alias then it defaults to `result`, which is the default alias for
  /// surrealdb results.
  ///
  /// With a response that looks like this:
  /// ```json
  /// [
  ///   { "result": [{ "username": "John", "id": "1" } }],
  ///   { "friends": [{ "username": "John", "id": "1" }]
  /// ]
  /// ```
  ///
  /// A call `response.get_nth_aliased_results(0, None)` would return `[{ "username": "John", "id": "1" } }]`.
  ///
  /// A call `response.get_nth_aliased_results(1, Some("friends"))` would return `[{ "username": "John", "id": "1" }]`
  ///
  pub fn get_nth_aliased_results(&self, n: usize, alias: Option<&str>) -> Option<&Vec<Value>> {
    // this array is if the query was composed of multiple queries separated by ;
    // otherwise it's a simple array filled with a single object
    match self.result.as_array() {
      Some(array) => match array.get(n) {
        Some(first) => {
          // now we're got an object that looks like:
          // { "result": [{ ... the object we're looking for .. }, ...] }
          // where result is either "result" by default or an alias given by the
          // query
          match first.get(alias.unwrap_or("result")) {
            Some(array_of_results) => match array_of_results.as_array() {
              Some(array_of_results) => Some(array_of_results),
              None => None,
            },
            None => None,
          }
        }
        None => None,
      },
      None => None,
    }
  }

  /// Equivalent to calling [`Self.get_aliased_results(None)`]
  pub fn get_results(&self) -> Option<&Vec<Value>> {
    self.get_aliased_results(None)
  }
}
