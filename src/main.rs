mod commands;
mod database;
mod util;

use std::{collections::HashSet, env, sync::Arc};

use database::*;
use commands::{ruoka::*, viikko::*, kuva::*, image_provider_msg::*, admin::*, ruokastats::*};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    builder::CreateEmbed,
    model::{event::ResumedEvent, gateway::Ready, channel::Message, channel::Reaction},
    prelude::*,
};
use arvio::Statistiikka;

use tracing::{error, info};

pub struct ShardManagerContainer;

pub struct RoleIDs {
    pub admin: u64,
    pub image_provider: u64,
    pub image_blog: u64,
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for RoleIDs {
    type Value = Arc<RoleIDs>;
}


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
    async fn message(&self, ctx:Context, msg:Message) {
        if msg.author.id.0 == ctx.data.read().await.get::<RoleIDs>().unwrap().clone().admin {
            handle_admin_message(ctx, msg).await.unwrap();
        }
        else if msg.author.id.0 == ctx.data.read().await.get::<RoleIDs>().unwrap().clone().image_provider {
            let blog_id = ctx.data.read().await.get::<RoleIDs>().unwrap().clone().image_blog;
            if msg.channel_id.0 == blog_id || blog_id == 0 {
                handle_image_provider_message(ctx, msg).await.unwrap();
            }
        }
    }
    async fn reaction_add(&self, ctx:Context, reaction:Reaction) {
        let bot_id = &ctx.http.get_current_application_info().await.unwrap().id;
        if reaction.user_id.unwrap().0 == bot_id.to_owned().0 {
            return
        }
        else if (1..6).contains(&reaction.emoji.to_string()[..1].parse::<i32>().unwrap()) {
            let message = &mut ctx.http.get_message(reaction.channel_id.0, reaction.message_id.0).await.unwrap();
            let kokoruoka = &message.embeds[0].fields[0].value;
            let ruoka = &kokoruoka[..match kokoruoka.find(',') {
                Some(n) => n,
                None => kokoruoka.len()
            }];
            info!("Reaction {} added by user `{}` to food: {}", &reaction.emoji, reaction.user_id.unwrap().0, ruoka);
            let db = ctx.data.read().await.get::<Database>().unwrap().clone();
            db.lisaa_arvio(reaction.user_id.unwrap().0, reaction.emoji.to_string()[..1].parse::<i32>().unwrap(), ruoka.to_string()).await;
            let stats: Statistiikka = db.anna_ruoan_statistiikka(ruoka.to_string()).await;
            let keskiarvo = match stats.keskiarvo {
                Some(s) => s.round(2).to_string(),
                None => "N/A".to_string(),
            };
            let mut orig_embed = message.embeds[0].clone();
            let orig_foodstring = &orig_embed.fields[0].value;
            let foodstring = format!("{}{}", &orig_foodstring[..orig_foodstring.find('(').unwrap()], format!("(:star:{}, {} arvostelija(a))", keskiarvo, stats.maara).as_str());
            orig_embed.fields[0].value = foodstring;
            message.edit(&ctx.http, |m| m.set_embed(CreateEmbed::from(orig_embed))).await.unwrap();
        }
    }
    async fn reaction_remove(&self, ctx:Context, reaction:Reaction) {
        let bot_id = &ctx.http.get_current_application_info().await.unwrap().id;
        if reaction.user_id.unwrap().0 == bot_id.to_owned().0 {
            return
        }
        else if (1..6).contains(&reaction.emoji.to_string()[..1].parse::<i32>().unwrap()) {
            let message = &mut ctx.http.get_message(reaction.channel_id.0, reaction.message_id.0).await.unwrap();
            let kokoruoka = &message.embeds[0].fields[0].value;
            let ruoka = &kokoruoka[..match kokoruoka.find(',') {
                Some(n) => n,
                None => kokoruoka.len()
            }];
            info!("Reaction {} removed by user `{}` from food: {}", &reaction.emoji, reaction.user_id.unwrap().0, ruoka);
            let db = ctx.data.read().await.get::<Database>().unwrap().clone();
            db.poista_arvio(reaction.user_id.unwrap().0, reaction.emoji.to_string()[..1].parse::<i32>().unwrap(), ruoka.to_string()).await;
            let stats: Statistiikka = db.anna_ruoan_statistiikka(ruoka.to_string()).await;
            let keskiarvo = match stats.keskiarvo {
                Some(s) => s.round(2).to_string(),
                None => "N/A".to_string(),
            };
            let mut orig_embed = message.embeds[0].clone();
            let orig_foodstring = &orig_embed.fields[0].value;
            let foodstring = format!("{}{}", &orig_foodstring[..orig_foodstring.find('(').unwrap()], format!("(:star:{}, {} arvostelija(a))", keskiarvo, stats.maara).as_str());
            orig_embed.fields[0].value = foodstring;
            message.edit(&ctx.http, |m| m.set_embed(CreateEmbed::from(orig_embed))).await.unwrap();
        }
    }
}

#[group]
#[commands(ruoka, viikko, kuva, ruokastats)]
struct General;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let database = Database::new().await;

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let admin = match env::var("ADMIN_ID") {
        Ok(i) => i.parse::<u64>().unwrap(),
        Err(_) => 0
    };

    let image_provider = match env::var("IMAGE_PROVIDER_ID") {
        Ok(i) => i.parse::<u64>().unwrap(),
        Err(_) => 0
    };

    let image_blog = match env::var("IMAGE_CHANNEL_ID") {
        Ok(i) => i.parse::<u64>().unwrap(),
        Err(_) => 0
    };

    let roleids = RoleIDs { admin, image_provider, image_blog};

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework =
        StandardFramework::new().configure(|c| c.owners(owners).prefix("!")).group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<RoleIDs>(Arc::new(roleids));
        data.insert::<Database>(Arc::new(database));
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
