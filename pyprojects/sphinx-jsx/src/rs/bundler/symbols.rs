use pyo3::FromPyObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromPyObject)]
pub struct WellKnownSymbols {
  // JSX Runtime
  pub jsx: String,

  pub jsxs: String,

  #[serde(rename = "Fragment")]
  pub fragment: String,

  // Lingui
  #[serde(rename = "Trans")]
  pub trans: String,

  #[serde(rename = "_")]
  pub gettext: String,

  // URL resolver
  pub url: String,
}
