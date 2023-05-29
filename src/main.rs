use std::collections::VecDeque;
use std::env::var;
use std::vec::Vec;

use rand::rngs::ThreadRng;
use rand::Rng;

use scraper::{Html, Selector};

use serenity::http::client::Http;
use serenity::model::id::ChannelId;

use tokio::time::{sleep, Duration};

async fn unusual(http: &Http, rng: &mut ThreadRng, chan: u64) {

    //
    // send a discord message announcing data collection
    //

    if let Err(why) = ChannelId(chan).say(
        http,
        "Collecting eldritch data..."
    ).await {
        println!("Unable to send Discord message: {:?}", why);
    }

    //
    // make the http request
    //

    let response;

    match reqwest::get(
        "https://en.wikipedia.org/wiki/Wikipedia:Unusual_Articles"
    ).await {
        Ok(r) => response = r,
        Err(why) => {
            println!("Unable to connect to Wikipedia: {:?}", why);
            return;
        }
    }

    //
    // throw away everything but the html
    //

    let html;

    match response.text().await {
        Ok(x) => html = x,
        Err(why) => {
            println!("Wikipedia sent back garbage! {:?}", why);
            return;
        }
    }

    //
    // create a list of all articles
    //

    let document = Html::parse_document(&html);

    let tbl_selector = Selector::parse(".wikitable")
        .expect("unreachable panic");

    let tr_selector = Selector::parse("tr")
        .expect("unreachable panic");

    let mut articles = Vec::new();

    for tbl in document.select(&tbl_selector) {
        for article in tbl.select(&tr_selector) {
            articles.push(article);
        }
    }

    //
    // select a random article
    //

    let article = articles[rng.gen_range(1..articles.len()) - 1];

    //
    // send the title and description on discord
    //

    let raw_text = article.text().collect::<Vec<_>>();
    let mut text = VecDeque::new();
    for i in raw_text {
        if i != "\n" {
            text.push_back(i);
        }
    }

    let mut msg;

    match text.pop_front() {
        Some(i) => msg = format!("*{}*: ", i.to_owned()),
        None => {
            println!("Error parsing Wikipedia!");
            return;
        },
    }

    while text.len() > 0 {
        match text.pop_front() {
            Some(i) => msg += i,
            None => std::unreachable!()
        }
    }

    if let Err(why) = ChannelId(chan).say(http, msg).await {
        println!("Unable to send Discord message: {:?}", why);
    }
}

#[tokio::main]
async fn main() {

    //
    // get the api key and channel from environment
    //

    let tok_res = var("key");
    let token = tok_res.expect(
        "Error reading \"key\" from environment."
    );

    let chan_res = var("chan");
    let chan = chan_res.expect(
        "Error reading \"chan\" from environment."
    ).parse::<u64>().unwrap();

    //
    // set up the bot
    //

    let http = Http::new(&token);

    let mut rng = rand::thread_rng();

    //
    // main loop
    //

    loop {
        unusual(&http, &mut rng, chan).await;

        //
        // wait a random amount of time between 6 and 72 hours
        //

        sleep(Duration::from_secs(
            rng.gen_range((60 * 60 * 6)..(60 * 60 * 72))
        )).await;
    }
}
