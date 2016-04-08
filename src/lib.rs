extern crate beautician;
#[macro_use]
extern crate hyper;
extern crate openssl;
extern crate url;
extern crate multipart;
extern crate rustc_serialize;

#[cfg(feature = "puppet-access")]
extern crate puppet_access;

#[macro_use]pub mod utils;
pub mod net;
pub mod config;
pub mod client;
pub mod admin;
