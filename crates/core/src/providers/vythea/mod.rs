//! Your Vythea account: the hosted pool, charged to your credit. Signed in
//! to once per computer (`auth.rs`), so there is nothing to paste. Its base
//! is the account API's, `auth::hosted_base`, which a test run can move.

use super::{AuthKind, Provider, ProviderInfo, Tier};

pub struct Vythea;

impl Provider for Vythea {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: crate::gateway::VYTHEA,
            base: "https://api.vythea.com/v1",
            blurb: "Your Vythea account. Hosted models, charged to your credit. No key.",
            docs: "https://vythea.com/docs/hosted-models",
            tier: Tier::Credit,
        }
    }

    fn auth(&self) -> AuthKind {
        AuthKind::Account
    }
}
