use pupcorn_core::{Database, Repo};

#[tokio::main]
async fn main() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let repo = Repo::new(db.pool.clone());

    let t = repo.insert_target("example.com", "https://example.com", Some("authorized")).await.unwrap();
    println!("target: {} {}", t.id, t.name);

    let s = repo.insert_session(t.id, "recon-1").await.unwrap();
    println!("session: {}", s.id);

    let f = repo
        .insert_finding(t.id, Some(s.id), pupcorn_core::FindingKind::Subdomain, "api.example.com", None, Some("https://api.example.com"))
        .await
        .unwrap();
    println!("finding: {} {}", f.id, f.title);

    repo.audit(Some(t.id), "scan_started", Some("http_probe")).await.unwrap();

    let targets = repo.list_targets().await.unwrap();
    println!("targets count: {}", targets.len());
    let findings = repo.list_findings(t.id).await.unwrap();
    println!("findings count: {}", findings.len());
}