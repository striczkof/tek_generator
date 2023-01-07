extern crate getopts;
use getopts::Options;
use ini::Ini;
use log::{info, warn};
use std::env;
// Log shit goes here
// Discord stuff
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready, gateway::GatewayIntents},
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
                println!("Error sending message: {:?}", why);
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
        println!("{} is connected!", ready.user.name);
    }
}


// Global variables
// Name of config file
const CONFIG_NAME: &str = "tek_generator";

// Check for INI files then load them if they exist. Make new ones if they don't.
fn load_config() -> Option<Ini> {
    // Look for config file in same directory and /etc directory if target is a unix system, use .conf extension
    #[cfg(target_family = "unix")]
        let conf_file = Ini::load_from_file(format!("/etc/{}.conf", CONFIG_NAME)).or_else(|_| Ini::load_from_file(format!("{}.conf", CONFIG_NAME)));
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
    print!("{}", opts.usage(&brief));
}

// DEMON MODE

#[tokio::main]
async fn main() {
    // Get options if any
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("t", "token", "Set the discord bot token", "TOKEN");
    opts.optflag("h", "help", "Print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };
    if matches.opt_present("h") {
        print_help_usage(&args[0], opts);
        return;
    }
    // Load config file
    let config = load_config().unwrap();
    // Set discord token if provided else use provided token from config file
    let token = if matches.opt_present("t") {
        matches.opt_str("t").unwrap()
    } else {
        config.get_from(Some("discord"), "token").unwrap().to_string()
    };
    println!("Token is {}", token);
    // Discord stuff here we go
    // Set intents, meant to be fine tuned later
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Error creating client");
    // Start an auto sharded client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
/*
fn main() {
    println!("Hello, world!");

}*/
