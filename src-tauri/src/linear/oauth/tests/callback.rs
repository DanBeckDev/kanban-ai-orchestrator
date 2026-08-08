use super::super::{loopback_callback_address, loopback_callback_path};
use super::{
    LinearOAuthConfiguration, LinearOAuthError, LinearOAuthService, MemoryCredentialStore,
    configuration,
};

#[test]
fn requires_a_nonblank_client_id_and_accepts_ipv6_loopback_callbacks() {
    let mut service = LinearOAuthService::new(MemoryCredentialStore::default());
    assert_eq!(
        service.begin(LinearOAuthConfiguration {
            client_id: " ".to_owned(),
            redirect_uri: "http://127.0.0.1:38471/linear/oauth/callback".to_owned(),
        }),
        Err(LinearOAuthError::MissingClientId)
    );

    assert!(
        service
            .begin(LinearOAuthConfiguration {
                client_id: "client-id".to_owned(),
                redirect_uri: "http://[::1]:38471/linear/oauth/callback".to_owned(),
            })
            .is_ok()
    );
}

#[test]
fn derives_the_exact_socket_and_callback_path_from_validated_loopback_uris() {
    let ipv4 = configuration();
    assert_eq!(
        loopback_callback_address(&ipv4).expect("IPv4 callback should resolve"),
        "127.0.0.1:38471"
            .parse()
            .expect("socket address should parse")
    );
    assert_eq!(
        loopback_callback_path(&ipv4).expect("IPv4 path should resolve"),
        "/linear/oauth/callback"
    );

    let ipv6 = LinearOAuthConfiguration {
        client_id: "client-id".to_owned(),
        redirect_uri: "http://[::1]:38471/linear/oauth/callback".to_owned(),
    };
    assert_eq!(
        loopback_callback_address(&ipv6).expect("IPv6 callback should resolve"),
        "[::1]:38471".parse().expect("socket address should parse")
    );
}
