pub mod db;
pub mod engine;
pub mod error;
pub mod models;
pub mod modules;
pub mod repo;
pub mod screenshot;
pub mod tool;

pub use db::Database;
pub use engine::{Approval, ToolEngine};
pub use error::{PupcornError, Result};
pub use models::*;
pub use modules::*;
pub use repo::Repo;
pub use screenshot::*;
pub use tool::{Tool, ToolOptions, ToolOutput};

/// Build a ToolEngine with all default modules registered.
pub fn default_engine() -> ToolEngine {
    let mut engine = ToolEngine::new();
    engine.register(SubdomainEnum);
    engine.register(PortScan);
    engine.register(HttpProbe);
    engine.register(TechFingerprint);
    engine.register(ContentDiscovery);
    engine.register(JsAnalysis);
    engine.register(ParamDiscovery);
    engine.register(DnsRecords);
    engine.register(WhoisRdap);
    engine.register(WaybackUrls);
    engine.register(IpInfo);
    engine.register(DorkGen);
    engine
}