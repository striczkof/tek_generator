extern crate getopts;
use getopts::Options;
use ini::Ini;
use std::env;
// Log shit goes here
use std::io::Write;
use flexi_logger::FlexiLoggerError::Log;
use chrono::Local;
use env_logger::Builder as LogBuilder;
use log::LevelFilter;
use log::{info, warn, error};
// Discord stuff
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::GatewayIntents, gateway::Ready},
};
use tokio::sync::mpsc;

// Temporary handler, yoinked from the serenity example TODO: Make this a proper handler
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                warn!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

// Global variables
// Name of config file
const CONFIG_NAME: &str = "tek_generator";

// Check for INI files then load them if they exist. Make new ones if they don't.
fn load_config() -> Option<Ini> {
    // Look for config file in same directory and /etc directory if target is a unix system, use .conf extension
    #[cfg(target_family = "unix")]
    let conf_file = Ini::load_from_file(format!("/etc/{}.conf", CONFIG_NAME))
        .or_else(|_| Ini::load_from_file(format!("{}.conf", CONFIG_NAME)));
    // Look for config file in same directory if target is not a unix system, use .ini extension
    #[cfg(not(target_family = "unix"))]
    let mut conf = Ini::load_from_file(format!("{}.ini", CONFIG_NAME));
    // If config file exists, load it
    return if conf_file.is_ok() {
        info!("I found a config file!");
        Some(conf_file.unwrap())
    } else {
        warn!("No config file found!");
        warn!("Creating a new config file. This will be empty, and the bot will not work unless you provided a token in options.");
        let mut conf = Ini::new();
        conf.with_section(Some("discord"))
            .set("token", "");
        // Generates config file with extension .conf or .ini depending on target
        #[cfg(target_family = "unix")]
        conf.write_to_file(format!("{}.conf", CONFIG_NAME)).unwrap();
        #[cfg(not(target_family = "unix"))]
        conf.write_to_file(format!("{}.ini", CONFIG_NAME)).unwrap();
        Some(conf)
    };
    // Debug code
    /*println!("Hi mom!");
    return false;*/
}

// Help menu for noobs
fn print_help_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

// ARK Tek Generator calculator function. Outputs seconds before generator runs out in double.
// 1 ELEMENT IS EQUALS 18 HOURS, SUCCESS!
fn calculate_tek_gen(range: f64, element: u32, element_shards: u32) -> f64 {
    // Combine element first, we use element shards (= 0.01 element) for calculation
    let total_element = (element * 100) + element_shards;
    println!("Total element is {}", total_element as f64);
    total_element as f64 * (648.0 / (1.0 + ((range - 1.0) * 0.33)))
}

// Main function
#[tokio::main]
async fn main() {
    // Get options if any
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("t", "token", "Set the discord bot token", "TOKEN");
    opts.optflag("h", "help", "Print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_help_usage(&args[0], opts);
        return;
    }
    // Set up loggerd
    LogBuilder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    // Load config file
    let config = load_config().unwrap();
    // Set discord token if provided else use provided token from config file
    let token = if matches.opt_present("t") {
        matches.opt_str("t").unwrap()
    } else {
        config
            .get_from(Some("discord"), "token")
            .unwrap()
            .to_string()
    };
    info!("Token is {}", token);
    // Exits program if token is empty
    if token.is_empty() {
        error!("Token is empty! Please provide a token in the config file or as an argument.");
        return;
    }
    // Discord stuff here we go
    // Set intents, meant to be fine tuned later
    let intents = GatewayIntents::default();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    // Start an auto sharded client
    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
/*
fn main() {
    println!("Hello, world!");

}*/
