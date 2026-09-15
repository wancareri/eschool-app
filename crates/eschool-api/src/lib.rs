//! Unofficial API client for diary.e-schools.by (e-schools.by school diary).
//!
//! # Quick start
//!
//! ```rust,no_run
//! use eschool_api::auth::Auth;
//!
//! // Login
//! let auth = Auth::new();
//! let token = auth.login_blocking("username", "password").unwrap();
//!
//! // Build a client and fetch data
//! let client = eschool_api::client::blocking::build_client(&token.access_token);
//! let user: eschool_api::entities::UserInfo =
//!     eschool_api::client::blocking::api_get(&client, eschool_api::client::endpoints::AUTH_ME).unwrap();
//! ```

pub mod auth;
pub mod client;
pub mod entities;
