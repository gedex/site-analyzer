use async_trait::async_trait;

use crate::{
    error::Result,
    types::{AnalysisResult, SiteData},
};

/// Supplemental detectors that add signals beyond the Wappalyzer fingerprints.
/// These cover page-level characteristics Wappalyzer doesn't track: cookie
/// attributes, HTTP version, compression, TLD, charset, structured-data flags, etc.
pub mod html;

#[async_trait]
pub trait Detector: Send + Sync {
    fn name(&self) -> &'static str;
    async fn detect(&self, data: &SiteData, result: &mut AnalysisResult) -> Result<()>;
}
