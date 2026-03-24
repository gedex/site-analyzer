use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use tracing::debug;

use crate::{
    error::Result,
    types::DnsRecords,
};

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    pub async fn new() -> Result<Self> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default());
        Ok(Self { resolver })
    }

    pub async fn resolve(&self, domain: &str) -> DnsRecords {
        let mut records = DnsRecords::default();

        // A records
        if let Ok(response) = self.resolver.lookup_ip(domain).await {
            for ip in response.iter() {
                if ip.is_ipv4() {
                    records.a_records.push(ip.to_string());
                } else {
                    records.aaaa_records.push(ip.to_string());
                }
            }
        }

        // MX records
        if let Ok(response) = self.resolver.mx_lookup(domain).await {
            for mx in response.iter() {
                records.mx_records.push(mx.exchange().to_string());
            }
        }

        // NS records
        if let Ok(response) = self.resolver.ns_lookup(domain).await {
            for ns in response.iter() {
                records.ns_records.push(ns.to_string());
            }
        }

        // TXT records
        if let Ok(response) = self.resolver.txt_lookup(domain).await {
            for txt in response.iter() {
                records
                    .txt_records
                    .push(txt.to_string());
            }
        }

        debug!(
            "DNS: {} A records, {} NS records for {}",
            records.a_records.len(),
            records.ns_records.len(),
            domain
        );

        records
    }
}
