use std::cell::RefCell;

use chrono::{Duration, Utc};

use super::{MemoryCredentialStore, credentials};
use crate::linear::LinearOAuthService;

#[test]
fn recognizes_only_targeted_or_broad_write_comment_permission() {
    let mut read_only = credentials(
        "client-id".to_owned(),
        "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        Utc::now() + Duration::hours(1),
    );
    let store = MemoryCredentialStore {
        credentials: RefCell::new(Some(read_only.clone())),
    };
    let service = LinearOAuthService::new(store);
    assert!(
        !service
            .comments_are_authorized()
            .expect("read-only credentials should remain readable")
    );

    read_only.scopes = vec!["write".to_owned()];
    let store = MemoryCredentialStore {
        credentials: RefCell::new(Some(read_only)),
    };
    let service = LinearOAuthService::new(store);
    assert!(
        service
            .comments_are_authorized()
            .expect("broad write credentials should remain readable")
    );
}
