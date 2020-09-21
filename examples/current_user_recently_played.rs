use rspotify::client::SpotifyBuilder;
use rspotify::oauth2::{CredentialsBuilder, OAuthBuilder};

#[tokio::main]
async fn main() {
    // Set RSPOTIFY_CLIENT_ID, RSPOTIFY_CLIENT_SECRET and
    // RSPOTIFY_REDIRECT_URI in an .env file or export them manually:
    //
    // export RSPOTIFY_CLIENT_ID="your client_id"
    // export RSPOTIFY_CLIENT_SECRET="secret"
    // export RSPOTIFY_REDIRECT_URI=your-direct-uri
    //
    // These will then be read with `from_env`.

    // Or set client_id and client_secret explictly:
    //
    // let creds = CredentialsBuilder::default()
    //     .client_id("this-is-my-client-id")
    //     .client_secret("this-is-my-client-secret")
    //     .build()
    //     .unwrap();
    let creds = CredentialsBuilder::from_env().build().unwrap();

    // Or set the redirect_uri explictly:
    //
    // let oauth = OAuthBuilder::default()
    //     .redirect_uri("http://localhost:8888/callback")
    //     .build()
    //     .unwrap();
    let oauth = OAuthBuilder::from_env()
        .scope("user-read-recently-played")
        .build()
        .unwrap();

    let mut spotify = SpotifyBuilder::default()
        .credentials(creds)
        .oauth(oauth)
        .build()
        .unwrap();

    // Obtaining the access token
    spotify.prompt_for_user_token().await.unwrap();

    // Running the requests
    let history = spotify.current_user_recently_played(10).await;

    println!("Response: {:?}", history);
}
