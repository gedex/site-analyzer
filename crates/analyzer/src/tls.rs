use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore},
    TlsConnector,
};
use tracing::warn;

use crate::{
    error::{AnalyzerError, Result},
    types::TlsInfo,
};

pub async fn fetch_tls_info(host: &str, port: u16) -> Option<TlsInfo> {
    match do_tls_handshake(host, port).await {
        Ok(info) => Some(info),
        Err(e) => {
            warn!("TLS info fetch failed for {}: {}", host, e);
            None
        }
    }
}

async fn do_tls_handshake(host: &str, port: u16) -> Result<TlsInfo> {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| AnalyzerError::TlsError(e.to_string()))?;

    let server_name = tokio_rustls::rustls::pki_types::ServerName::try_from(host.to_string())
        .map_err(|e| AnalyzerError::TlsError(e.to_string()))?;

    let tls_stream = connector
        .connect(server_name, stream)
        .await
        .map_err(|e| AnalyzerError::TlsError(e.to_string()))?;

    let (_, session) = tls_stream.get_ref();

    let mut issuer_org: Option<String> = None;
    let mut issuer_cn:  Option<String> = None;
    let mut subject_cn: Option<String> = None;

    if let Some(certs) = session.peer_certificates() {
        if let Some(cert_der) = certs.first() {
            let parsed = x509_heuristic_parse(cert_der.as_ref());
            issuer_org = parsed.0;
            issuer_cn  = parsed.1;
            subject_cn = parsed.2;
        }
    }

    Ok(TlsInfo {
        issuer_org,
        issuer_cn,
        subject_cn,
        not_after: None,
    })
}

/// Minimal heuristic DER scanner.
/// Finds OID 2.5.4.3 (CommonName = 55 04 03) and 2.5.4.10 (Org = 55 04 0A).
/// Returns (issuer_org, issuer_cn, subject_cn).
fn x509_heuristic_parse(der: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    const CN_OID:  &[u8] = &[0x55, 0x04, 0x03];
    const ORG_OID: &[u8] = &[0x55, 0x04, 0x0a];

    let mut issuer_org: Option<String> = None;
    let mut issuer_cn:  Option<String> = None;
    let mut subject_cn: Option<String> = None;
    let mut cn_count = 0usize;

    let mut i = 0;
    while i + 3 < der.len() {
        if der[i..].starts_with(CN_OID) {
            if let Some(val) = read_der_string(der, i + CN_OID.len()) {
                match cn_count {
                    0 => issuer_cn  = Some(val),
                    1 => subject_cn = Some(val),
                    _ => {}
                }
                cn_count += 1;
            }
        } else if der[i..].starts_with(ORG_OID) && issuer_org.is_none() {
            issuer_org = read_der_string(der, i + ORG_OID.len());
        }
        i += 1;
    }

    (issuer_org, issuer_cn, subject_cn)
}

/// Read the next DER-encoded string tag at `pos`.
/// Accepted tags: UTF8String(0x0C), PrintableString(0x13), IA5String(0x16),
///                TeletexString(0x14), BMPString(0x1E).
fn read_der_string(der: &[u8], pos: usize) -> Option<String> {
    if pos + 2 > der.len() {
        return None;
    }
    if ![0x0C, 0x13, 0x16, 0x14, 0x1E].contains(&der[pos]) {
        return None;
    }
    let len = der[pos + 1] as usize;
    if pos + 2 + len > der.len() {
        return None;
    }
    String::from_utf8(der[pos + 2..pos + 2 + len].to_vec()).ok()
}
