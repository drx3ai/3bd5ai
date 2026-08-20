use super::*;

#[test]
fn classifies_personal_access_tokens_by_prefix() {
    assert!(matches!(
        classify_bd3_access_token("at-example"),
        Bd3AccessToken::PersonalAccessToken("at-example")
    ));
    assert!(matches!(
        classify_bd3_access_token("header.payload.signature"),
        Bd3AccessToken::AgentIdentityJwt("header.payload.signature")
    ));
}
