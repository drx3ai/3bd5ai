const PERSONAL_ACCESS_TOKEN_PREFIX: &str = "at-";

pub(super) enum Bd3AccessToken<'a> {
    PersonalAccessToken(&'a str),
    AgentIdentityJwt(&'a str),
}

pub(super) fn classify_bd3_access_token(access_token: &str) -> Bd3AccessToken<'_> {
    if access_token.starts_with(PERSONAL_ACCESS_TOKEN_PREFIX) {
        Bd3AccessToken::PersonalAccessToken(access_token)
    } else {
        Bd3AccessToken::AgentIdentityJwt(access_token)
    }
}

#[cfg(test)]
#[path = "access_token_tests.rs"]
mod tests;
