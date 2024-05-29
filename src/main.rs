use dotenv::dotenv;
use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{stdin, stdout, Write};

#[derive(Deserialize, Debug)]
struct OpenAIChoices {
    text: String,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OpenAIChoices>,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    prompt: String,
    max_token: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let https = HttpsConnector::new();
    let client = Client::builder().build(https);

    let uri = "https://api.openai.com/v1/engines/davinci-002/completions";

    let preamble = "Generate an SQL code for the given statement: ";
    let openai_token: String = env::var("OAI_TOKEN").unwrap();
    let auth_header_val = format!("Bearer {}", openai_token); // Added space after Bearer
    println!("{esc}c", esc = 27 as char);

    loop {
        print!(">");
        stdout().flush().unwrap();
        let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");
        println!("");

        let sp = Spinner::new(&Spinners::Dots12, "\t\tBot is thinking....".into());
        let openai_request = OpenAIRequest {
            prompt: format!("{} {}", preamble, user_text),
            max_token: 1000,
        };

        let body = Body::from(serde_json::to_vec(&openai_request)?);
        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", &auth_header_val)
            .body(body)
            .unwrap();

        match client.request(req).await {
            Ok(res) => {
                let body = hyper::body::aggregate(res).await?;
                let body_text =
                    std::str::from_utf8(body.chunk()).expect("Failed to read body text");

                match serde_json::from_str::<OpenAIResponse>(body_text) {
                    Ok(json) => {
                        sp.stop();
                        println!("");
                        print!("{}", json.choices[0].text);
                    }
                    Err(e) => {
                        sp.stop();
                        eprintln!(
                            "Failed to parse response: {}\nResponse body: {}",
                            e, body_text
                        );
                    }
                }
            }
            Err(e) => {
                sp.stop();
                eprintln!("Request failed: {}", e);
            }
        }
    }
    Ok(())
}
