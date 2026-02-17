use fluxer::prelude::*;
use async_trait::async_trait;
use std::env;

const CUSTOM_API_URL: Option<&str> = None;

struct TestBot;

#[async_trait]
impl EventHandler for TestBot {
    async fn on_ready(&self, _ctx: Context, ready: Ready) {
        println!("Login Successful");
        let disc = ready.user.discriminator.clone().unwrap_or_else(|| "0000".to_string());
        println!("User: {}#{}", ready.user.username, disc);
        println!("Session id: {}", ready.session_id);
    }

    async fn on_guild_create(&self, _ctx: Context, guild: Guild) {
        let name = guild.name.clone().unwrap_or_else(|| "Unknown Guild".to_string());
        println!("Guild Loaded: {} ({})", name, guild.id);
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        if msg.author.bot.unwrap_or(false) {
            return;
        }

        let content = msg.content.clone();
        let channel_id = msg.channel_id.clone();
        
        let send = |text: &str| {
            let http = ctx.http.clone();
            let cid = channel_id.clone();
            let txt = text.to_string();
            async move {
                http.send_message(&cid, &txt).await
            }
        };

        if content == "!test" {
            if let Err(e) = send("fluxer-rs bot is working").await {
                println!("Failed: {:?}", e);
            }
        }
        else if content == "!me" {
            match ctx.http.get_me().await {
                Ok(user) => {
                    let response = format!("I am: {} (ID: <@{}>)", user.username, user.id);
                    let _ = send(&response).await;
                }
                Err(e) => println!("get_me failed: {:?}", e),
            }
        }
        else if content == "!guild" {
            if let Some(guild_id) = &msg.guild_id {
                match ctx.http.get_guild(guild_id).await {
                    Ok(guild) => {
                        let g_name = guild.name.unwrap_or_else(|| "Unknown".to_string());
                        let owner = guild.owner_id.unwrap_or_else(|| "Unknown".to_string());
                        
                        let response = format!("This guild is '{}' owned by <@{}>", g_name, owner);
                        let _ = send(&response).await;
                    }
                    Err(e) => {
                        let _ = send(&format!("Failed to fetch guild: {:?}", e)).await;
                    }
                }
            } else {
                let _ = send("This command only works inside a server").await;
            }
        }
        else if content.starts_with("!create_channel ") {
            let new_name = &content[16..];
            if let Some(guild_id) = &msg.guild_id {
                match ctx.http.create_channel(guild_id, new_name, ChannelType::Text).await {
                    Ok(channel) => {
                        let c_name = channel.name.unwrap_or_else(|| "new-channel".to_string());
                        let _ = send(&format!("Created channel '{}' (ID: {})", c_name, channel.id)).await;
                    }
                    Err(e) => {
                        let _ = send(&format!("Failed to create channel: {:?}", e)).await;
                    }
                }
            }
        }
        else if content == "!delete" {
            match send("delete command").await {
                Ok(sent_msg) => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    if let Err(e) = ctx.http.delete_message(&sent_msg.channel_id, &sent_msg.id).await {
                        let _ = send(&format!("Failed to delete: {:?}", e)).await;
                    } else {
                        println!("Message deleted");
                    }
                }
                Err(_) => {}
            }
        }
        else if content == "!ban" {
            if let Some(guild_id) = &msg.guild_id {
                let _ = send("banning the message author").await;
                match ctx.http.ban_member(guild_id, &msg.author.id, "Requested via !ban command").await {
                    Ok(_) => println!("Ban command sent"),
                    Err(e) => {
                        let _ = send(&format!("Failed to ban: {:?}", e)).await;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("FLUXERTOKEN").expect("Please set FLUXERTOKEN environment variable");

    println!("Starting Bot Test");

    let mut builder = Client::builder(token)
        .event_handler(TestBot);

    if let Some(url) = CUSTOM_API_URL {
        println!("Using Custom API URL: {}", url);
        builder = builder.api_url(url);
    }

    let mut client = builder.build();

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}