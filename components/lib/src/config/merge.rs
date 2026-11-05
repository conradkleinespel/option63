//! Recursive merging logic for combining layered configuration files.

use crate::config::{
    CarddavConfig, ClientConfig, Config, ProxyConfig, ProxyDownstreamConfig, ProxyUpstreamConfig,
    WebConfig, WebDatabaseConfig, WebDatabaseTlsConfig,
};

pub trait Merge {
    fn merge(self, incoming: Self) -> Self;
}

impl<T> Merge for Option<T> {
    fn merge(self, incoming: Self) -> Self {
        incoming.or(self)
    }
}

macro_rules! impl_merge {
    ($type:ident { $($field:ident),* $(,)? }) => {
        impl Merge for $type {
            fn merge(self, incoming: Self) -> Self {
                Self {
                    $( $field: self.$field.merge(incoming.$field), )*
                }
            }
        }
    };
}

impl_merge!(CarddavConfig {
    client,
    proxy,
    dangerous_log_request_response_body,
});

impl_merge!(ClientConfig {
    url,
    username,
    password,
    dangerous_allow_plain_text_http,
});

impl_merge!(ProxyConfig {
    upstream,
    downstream,
});

impl_merge!(ProxyUpstreamConfig {
    url,
    username,
    password,
    dangerous_allow_plain_text_http,
});

impl_merge!(ProxyDownstreamConfig {
    listen,
    username,
    password,
    read_only,
    tls_cert_path,
    tls_key_path,
    dangerous_trust_forwarded_proto,
    dangerous_allow_acl_passthrough,
});

impl_merge!(WebConfig { listen, database });

impl_merge!(WebDatabaseConfig {
    host,
    port,
    name,
    username,
    password,
    tls,
});

impl_merge!(WebDatabaseTlsConfig { enabled });

fn merge_role<T: Merge>(base: Option<T>, incoming: Option<T>) -> Option<T> {
    match (base, incoming) {
        (Some(base), Some(incoming)) => Some(base.merge(incoming)),
        (None, incoming) => incoming,
        (base, None) => base,
    }
}

impl Merge for Config {
    fn merge(self, incoming: Self) -> Self {
        Self {
            carddav: merge_role(self.carddav, incoming.carddav),
            web: merge_role(self.web, incoming.web),
        }
    }
}

pub fn merge(base: Config, incoming: Config) -> Config {
    base.merge(incoming)
}
