use chrono::prelude::*;
use chrono::Duration;
use rspotify::{
    prelude::*, scopes, AuthCodeSpotify, ClientCredsSpotify, Config, Credentials, OAuth, Token,
};
use std::{collections::HashMap, fs, io::Read, path::PathBuf};
use url::Url;
use wasm_bindgen_test::*;

#[test]
#[wasm_bindgen_test]
fn test_get_authorize_url() {
    let oauth = OAuth {
        state: "fdsafdsfa".to_owned(),
        redirect_uri: "localhost".to_owned(),
        scopes: scopes!("playlist-read-private"),
        ..Default::default()
    };
    let creds = Credentials::new("this-is-my-client-id", "this-is-my-client-secret");

    let spotify = AuthCodeSpotify::new(creds, oauth);

    let authorize_url = spotify.get_authorize_url(false).unwrap();
    let hash_query: HashMap<_, _> = Url::parse(&authorize_url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();

    assert_eq!(hash_query.get("client_id").unwrap(), "this-is-my-client-id");
    assert_eq!(hash_query.get("response_type").unwrap(), "code");
    assert_eq!(hash_query.get("redirect_uri").unwrap(), "localhost");
    assert_eq!(hash_query.get("scope").unwrap(), "playlist-read-private");
    assert_eq!(hash_query.get("state").unwrap(), "fdsafdsfa");
}

#[maybe_async::test(feature = "__sync", async(feature = "__async", tokio::test))]
async fn test_read_token_cache() {
    let expires_in = Duration::try_seconds(3600).unwrap();
    let expires_at = Some(Utc::now() + expires_in);
    let scopes = scopes!("playlist-read-private", "playlist-read-collaborative");

    let tok = Token {
        expires_in,
        expires_at,
        access_token: "test-access_token".to_owned(),
        scopes: scopes.clone(),
        refresh_token: Some("...".to_owned()),
    };

    let config = Config {
        token_cached: true,
        cache_path: PathBuf::from(".test_read_token_cache.json"),
        ..Default::default()
    };
    let mut predefined_spotify = ClientCredsSpotify::from_token(tok);
    predefined_spotify.config = config.clone();

    // write token data to cache_path
    predefined_spotify.write_token_cache().await.unwrap();
    assert!(predefined_spotify.config.cache_path.exists());

    let mut spotify = ClientCredsSpotify::default();
    spotify.config = config;

    // read token from cache file
    let tok_from_file = spotify.read_token_cache().await.unwrap().unwrap();
    assert_eq!(tok_from_file.scopes, scopes);
    assert_eq!(tok_from_file.refresh_token.unwrap(), "...");
    assert_eq!(
        tok_from_file.expires_in,
        Duration::try_seconds(3600).unwrap()
    );
    assert_eq!(tok_from_file.expires_at, expires_at);

    // delete cache file in the end
    fs::remove_file(&spotify.config.cache_path).unwrap();
}

#[maybe_async::test(feature = "__sync", async(feature = "__async", tokio::test))]
async fn test_write_token() {
    let now = Utc::now();
    let scopes = scopes!("playlist-read-private", "playlist-read-collaborative");

    let tok = Token {
        access_token: "test-access_token".to_owned(),
        expires_in: Duration::try_seconds(3600).unwrap(),
        expires_at: Some(now),
        scopes: scopes.clone(),
        refresh_token: Some("...".to_owned()),
    };

    let config = Config {
        token_cached: true,
        cache_path: PathBuf::from(".test_write_token_cache.json"),
        ..Default::default()
    };
    let mut spotify = ClientCredsSpotify::from_token(tok.clone());
    spotify.config = config;

    let tok_str = serde_json::to_string(&tok).unwrap();
    spotify.write_token_cache().await.unwrap();

    let mut file = fs::File::open(&spotify.config.cache_path).unwrap();
    let mut tok_str_file = String::new();
    file.read_to_string(&mut tok_str_file).unwrap();

    assert_eq!(tok_str, tok_str_file);
    let tok_from_file: Token = serde_json::from_str(&tok_str_file).unwrap();
    assert_eq!(tok_from_file.scopes, scopes);
    assert_eq!(
        tok_from_file.expires_in,
        Duration::try_seconds(3600).unwrap()
    );
    assert_eq!(tok_from_file.expires_at.unwrap(), now);

    // delete cache file in the end
    fs::remove_file(&spotify.config.cache_path).unwrap();
}

#[test]
#[wasm_bindgen_test]
fn test_token_is_expired() {
    let expires_in = Duration::try_seconds(20).unwrap();
    let tok = Token {
        scopes: scopes!("playlist-read-private", "playlist-read-collaborative"),
        access_token: "test-access_token".to_owned(),
        expires_in,
        expires_at: Some(Utc::now() + expires_in),
        refresh_token: Some("...".to_owned()),
    };
    assert!(!tok.is_expired());

    let expires_in = Duration::try_seconds(3).unwrap(); // There's a margin of 10 seconds
    let tok = Token {
        scopes: scopes!("playlist-read-private", "playlist-read-collaborative"),
        access_token: "test-access_token".to_owned(),
        expires_in,
        expires_at: Some(Utc::now() + expires_in),
        refresh_token: Some("...".to_owned()),
    };
    assert!(tok.is_expired());
}

#[test]
#[wasm_bindgen_test]
fn test_parse_response_code() {
    // A random state is generated by default
    let spotify = AuthCodeSpotify::default();

    // No `code` parameter
    let url = "http://localhost:8888/callback";
    let code = spotify.parse_response_code(url);
    assert_eq!(code, None);

    // No `state` parameter
    let url = "http://localhost:8888/callback?code=AQD0yXvFEOvw";
    let code = spotify.parse_response_code(url);
    assert_eq!(code, None);

    // The `state` is not the expected one
    let url = "http://localhost:8888/callback?code=AQD0yXvFEOvw?state=abc";
    let code = spotify.parse_response_code(url);
    assert_eq!(code, None);

    // Both parameters, and the state is the same, so it should work
    let url = format!(
        "http://localhost:8888/callback?code=AQD0yXvFEOvw&state={}",
        spotify.oauth.state
    );
    let code = spotify.parse_response_code(&url);
    assert_eq!(code, Some("AQD0yXvFEOvw".to_string()));

    // Works both ways
    let url = format!(
        "http://localhost:8888/callback?state={}&code=AQD0yXvFEOvw",
        spotify.oauth.state
    );
    let code = spotify.parse_response_code(&url);
    assert_eq!(code, Some("AQD0yXvFEOvw".to_string()));
}
