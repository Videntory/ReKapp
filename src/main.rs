#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate regex;
extern crate async_compression;
extern crate rand;

use std::error::Error;
use hyper::body;
use hyper::client::connect::Connect;
use hyper::http::HeaderValue;
use hyper::{Body, Method, Client, Request};
use hyper_tls::HttpsConnector;
use tokio;
use tokio::io::AsyncWriteExt as _;
use regex::Regex;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
struct StreamPlaybackAccessTokenResponse {
    data: StreamPlaybackAccessTokenData
}

#[derive(Deserialize)]
struct StreamPlaybackAccessTokenData {
    #[allow(non_snake_case)]
    streamPlaybackAccessToken: StreamPlaybackAccessToken
}

#[derive(Deserialize)]
struct StreamPlaybackAccessToken {
    value: String,
    signature: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);
    // https://gql.twitch.tv/gql
    from_live_channel(client, "").await?;
    // from_video_id(client, "809855459").await?;

    // if let set_cookies = response.headers().keys() {
    //     let mut iterator = set_cookies.iter();
    //     while let Some(set_cookie) = iterator.next() {
    //         println!("Set-Cookie: {}", String::from_utf8(set_cookie.as_bytes().to_vec()).expect("Set-Cookie was not valid utf-8"));
    //     }
    // }

    // let body_bytes = body::to_bytes(response.into_body()).await?;
    // let body = String::from_utf8(body_bytes.to_vec()).expect("response was not valid utf-8");
    // println!("Body: {}", body);
    Ok(())
}

async fn from_live_channel<C>(client: Client<C, Body>, channel_name: &str) -> Result<(), Box<dyn Error + Send + Sync>>
where
    C: Connect + Send + Clone + Sync + 'static
{
    println!("from_live_channel");
    // TODO: check security of using {channel_name} in format!(url + channel_name) GET request
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("https://www.twitch.tv/{}", channel_name))
        .header("Host", "www.twitch.tv")
        .header("Connection", "keep-alive")
        .header("Pragma", "no-cache")
        .header("Cache-Control", "no-cache")
        .header("DNT", "1")
        .header("Upgrade-Insecure-Request", "1")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-User", "?1")
        .header("Sec-Fetch-Dest", "document")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cookie", "twitch.lohp.countryCode=NL")
        .body(Body::empty())
        .unwrap();
        
    let response = client.request(request).await?;
    println!("Status: {}", response.status());

    let mut server_session_id: String = String::new(); // FIXME: how can I just allocate a fixed sized Box<str>?
    let mut device_id: String = String::new();
    {
        let mut found_server_session_id = false;
        let mut found_device_id = false;
        lazy_static! {
            static ref REGEX1: Regex = Regex::new(r#"(?:server_session_id=(\w+);)"#).unwrap(); // TODO: should be one regex, but lazy
            static ref REGEX2: Regex = Regex::new(r#"(?:unique_id=(\w+);)"#).unwrap();
        }
        let cookies = response.headers().get_all("set-cookie").iter().any(|set_cookie: &HeaderValue| -> bool {
            let text = set_cookie.to_str().expect("set_cookie was not valid ascii");
    
            if found_server_session_id == false {
                &REGEX1.captures_iter(text).next().and_then(|capture: regex::Captures| -> Option<()> {
                    println!("server_session_id: {}", &capture[1]);
                    server_session_id = String::from(&capture[1]);
                    found_server_session_id = true;
                    None
                });
            }
    
            if found_device_id == false {
                &REGEX2.captures_iter(text).next().and_then(|capture: regex::Captures| -> Option<()> {
                    println!("unique_id: {}", &capture[1]);
                    device_id = String::from(&capture[1]);
                    found_device_id = true;
                    None
                });
            }

            found_server_session_id && found_device_id
        });
        if cookies == false {
            panic!("unable to get cookies");
        }
    }

    let body_bytes = body::to_bytes(response.into_body()).await?;
    let mut decoder = async_compression::tokio_02::write::BrotliDecoder::new(Vec::new());
    decoder.write_all(&body_bytes.to_vec()).await?;
    decoder.shutdown().await?;
    let decompressed_data = decoder.into_inner();
    let body = String::from_utf8(decompressed_data).expect("response was not valid utf-8");
    
    lazy_static! {
        static ref REGEX3: Regex = Regex::new(r#"(?:"Client-ID":"(\w+)")"#).unwrap();
    }
    let client_id = &REGEX3.captures_iter(&body).next().expect("client-id not found")[1];
    println!("Client-ID: {}", client_id);
    
    let gql_options_request = Request::builder()
        .method(Method::OPTIONS)
        .uri("https://gql.twitch.tv/gql")
        .header("Accept", "*/*")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "en-US,en;q=9.0")
        .header("Accept-Control-Request-Headers", "authorization,client-id,device-id")
        .header("Accept-Control-Request-Method", "POST")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("Host", "gql.twitch.tv")
        .header("Origin", "https://www.twitch.tv")
        .header("Pragma", "no-cache")
        .header("Referer", "https://www.twitch.tv/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-site")
        //.header("User-Agent", "None/1.0 (None)")
        .body(Body::empty())
        .unwrap();

    let gql_options_response = client.request(gql_options_request).await?;
    let query = format!(r#"{{"operationName":"PlaybackAccessToken_Template","query":"query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $vodID: ID!, $isVod: Boolean!, $playerType: String!) {{  streamPlaybackAccessToken(channelName: $login, params: {{platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}}) @include(if: $isLive) {{    value    signature    __typename  }}  videoPlaybackAccessToken(id: $vodID, params: {{platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}}) @include(if: $isVod) {{    value    signature    __typename  }}}}","variables":{{"isLive":true,"login":"{}","isVod":false,"vodID":"","playerType":"site"}}}}"#, channel_name);
    let gql_playback_access_token_template_request = Request::builder()
        .method(Method::POST)
        .uri("https://gql.twitch.tv/gql")
        .header("Accept", "*/*")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "en-US")
        .header("Authorization", "undefined")
        .header("Cache-Control", "no-cache")
        .header("Client-ID", client_id)
        .header("Connection", "keep-alive")
        .header("Content-Length", query.len())
        .header("Content-Type", "text/plain; charset=UTF-8")
        .header("Device-ID", device_id)
        .header("DNT", "1")
        .header("Host", "gql.twitch.tv")
        .header("Origin", "https://www.twitch.tv")
        .header("Pragma", "no-cache")
        .header("Referer", "https://www.twitch.tv/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-site")
        .body(Body::from(query))
        .unwrap();
    println!("request: gql playback access token template");
    let new_response = client.request(gql_playback_access_token_template_request).await?;

    println!("convert to bytes: gql playback access token response");
    let new_body_bytes = body::to_bytes(new_response.into_body()).await?;
    let new_body = String::from_utf8(new_body_bytes.to_vec()).expect("response was not valid utf-8");
    println!("convert from json: stream playback access token response");
    let stream_playback_access_token_response: StreamPlaybackAccessTokenResponse =  serde_json::from_str(new_body.as_str())?;

    let mut rng = rand::thread_rng();
    let p = rng.gen_range(0, 999999 + 1);
    //let p = (9999999 * rng.gen()).floor();
    // "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".replace(/[xy]/g, (function(e) {
    //     var t = 16 * Math.random() | 0;
    //     return ("x" === e ? t : 3 & t | 8).toString(16)
    // }))
    const CAPACITY: usize = 32;
    const HEXADECIMALS: &[u8] = b"0123456789abcdef";
    let mut player_session_id = String::with_capacity(CAPACITY);
    for i in 0..CAPACITY {
        player_session_id.push(HEXADECIMALS[rng.gen_range(0, 16)] as char);
    }
    
    
    
    let mut channel_m3u8_url = url::Url::parse(format!("https://usher.ttvnw.net/api/channel/hls/{}.m3u8", channel_name).as_str()).unwrap();
    channel_m3u8_url.query_pairs_mut()
        .append_pair("allow_source", "true")
        .append_pair("fast_bread", "true")
        .append_pair("playlist_include_framerate", "true")
        .append_pair("reassignments_supported", "true")
        .append_pair("sig", stream_playback_access_token_response.data.streamPlaybackAccessToken.signature.as_str()) // from previous content
        .append_pair("token", stream_playback_access_token_response.data.streamPlaybackAccessToken.value.replace("\\", "").as_str())
        .append_pair("player_backend", "mediaplayer")
        .append_pair("p", p.to_string().as_str()) // client-side generated
        .append_pair("supported_codecs", "vp09,avc1")
        .append_pair("play_session_id", player_session_id.as_str()); // client-side generated
    println!("nauth_token: {}", stream_playback_access_token_response.data.streamPlaybackAccessToken.value.replace("\\", "").as_str());
    let channel_m3u8_request = Request::builder()
        .method(Method::GET)
        .uri(channel_m3u8_url.as_str())
        .header("Accept", "application/x-mpegURL, application/vnd.apple.mpegurl, application/json, text/plain")
        .header("DNT", "1")
        .header("Referer", "")
        .body(Body::empty())
        .unwrap();

    let channel_m3u8_response = client.request(channel_m3u8_request).await?;
    let channel_m3u8_body_bytes = body::to_bytes(channel_m3u8_response.into_body()).await?;
    // let mut gzip_decoder = async_compression::tokio_02::write::GzipDecoder::new(Vec::new());
    // gzip_decoder.write_all(&channel_m3u8_body_bytes.to_vec()).await?;
    // gzip_decoder.shutdown().await?;
    // let channel_m3u8_body_bytes_decompressed = gzip_decoder.into_inner();
    let channel_m3u8_body = String::from_utf8(channel_m3u8_body_bytes.to_vec()).expect("response was not valid utf-8");
    
    println!("");
    //println!("channel_m3u8_body:");
    //println!("{}", channel_m3u8_body);

    // lazy_static! {
    //     static ref REGEX4: Regex = Regex::new(r#"/(?:NAME=".*source.*"(?:.*\n#)*.*\n(https://.*.m3u8))/gi"#).unwrap();
    // }
    let regex4 = Regex::new(r#"(?:NAME=".*source.*"(?:.*\n#)*.*\n(https://.*.m3u8))"#).unwrap();
    
    let text_test = match regex4.captures(channel_m3u8_body.as_str()) {
        None => "",
        Some(captures) => {
            println!("capture found");
            captures.get(1).map_or("", |m|m.as_str())
        }
    };
    
    println!("text_test: {}", text_test);
    

    Ok(())
}

async fn from_video_id<C>(client: Client<C, Body>, id: &str) -> Result<(), Box<dyn Error + Send + Sync>>
where 
    C: Connect + Send + Clone + Sync + 'static
{
    println!("from_video_id");
    // TODO: check security of using {id} in format!(url + id) GET request
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("https://www.twitch.tv/videos/{}", id))
        .header("Host", "www.twitch.tv")
        .header("Connection", "keep-alive")
        .header("Pragma", "no-cache")
        .header("Cache-Control", "no-cache")
        .header("DNT", "1")
        .header("Upgrade-Insecure-Request", "1")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-User", "?1")
        .header("Sec-Fetch-Dest", "document")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cookie", "twitch.lohp.countryCode=NL")
        .body(Body::empty())
        .unwrap();
        
    let response = client.request(request).await?;
    println!("Status: {}", response.status());

    let server_session_id;
    if let Some(set_cookie) = response.headers().get("set-cookie") {
        let regex = Regex::new(r"(?:server_session_id=(\w+);)").unwrap();
        if let Some(capture) = regex.captures_iter(set_cookie.to_str().expect("set_cookie was not valid ascii")).next() {
            server_session_id = &capture[1];
            println!("server_session_id; {}", server_session_id);
        }
    }

    Ok(())
}
