#![feature(iter_intersperse)]

use std::env;

use dicemind::interpreter::StandardVerboseRoller;
use dicemind::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let Ok(expr) = parse(&msg.content) else {
            println!("Message was not parsed");
            return;
        };

        let mut roller = StandardVerboseRoller::new();
        match roller.roll(expr.clone()) {
            Ok(rolled) => {
                let total = rolled.total().to_markdown_string();
                let annotations = rolled.annotated_results().collect::<Vec<_>>();
                let subrolls = annotations
                    .into_iter()
                    .map(|(name, (expr, result))| {
                        format!("[{name}] {expr} = {}", result.to_markdown_string())
                    })
                    .intersperse('\n'.to_string())
                    .collect::<String>();
                msg.reply(&ctx.http, format!("{expr} = {total}\n{subrolls}")).await.unwrap();
            }
            Err(why) => {
                msg.reply(&ctx.http, format!("Roll failed: {}", why.to_string()))
                    .await
                    .unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
