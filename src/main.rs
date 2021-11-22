mod commands;
mod database;
mod util;

use std::{collections::HashSet, env, fs::File, io::BufRead, io::BufReader, sync::Arc};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    admin::*, food::*, foodstats::*, image::*, image_provider_msg::*, reactions::*, week::*,
};
use database::*;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{channel::Message, channel::Reaction, event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use tracing::{error, info};

pub struct ShardManagerContainer;

pub struct RoleIDs {
    pub admin: Vec<u64>,
    pub image_provider: Vec<u64>,
    pub image_blog: u64,
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for RoleIDs {
    type Value = RoleIDs;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let data = ctx.data.read().await;
        let roles = data.get::<RoleIDs>().unwrap();
        for admin in &roles.admin {
            let adminuser = ctx.http.get_user(*admin).await.unwrap();
            adminuser
                .dm(&ctx.http, |m| m.content("Ruokabotti is up and running!"))
                .await
                .unwrap();
        }
        info!("Connected as {}", ready.user.name);
    }
    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if ctx
            .data
            .read()
            .await
            .get::<RoleIDs>()
            .unwrap()
            .clone()
            .admin
            .contains(&msg.author.id.0)
        {
            handle_admin_message(ctx.clone(), msg.clone())
                .await
                .unwrap();
        }
        if ctx
            .data
            .read()
            .await
            .get::<RoleIDs>()
            .unwrap()
            .clone()
            .image_provider
            .contains(&msg.author.id.0)
        {
            let blog_id = ctx
                .data
                .read()
                .await
                .get::<RoleIDs>()
                .unwrap()
                .clone()
                .image_blog;
            if msg.channel_id.0 == blog_id || blog_id == 0 {
                handle_image_provider_message(ctx, msg).await.unwrap();
            }
        }
    }
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let bot_id = &ctx.http.get_current_application_info().await.unwrap().id;
        if reaction.message(&ctx.http).await.unwrap().author.id != bot_id.to_owned().0 {
            return;
        }
        if reaction.user_id.unwrap().0 == bot_id.to_owned().0 {
            return;
        }
        food_reaction(&ctx, &reaction, true).await;
    }
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let bot_id = &ctx.http.get_current_application_info().await.unwrap().id;
        if reaction.message(&ctx.http).await.unwrap().author.id != bot_id.to_owned().0 {
            return;
        }
        if reaction.user_id.unwrap().0 == bot_id.to_owned().0 {
            return;
        }
        food_reaction(&ctx, &reaction, false).await;
    }
}

#[group]
#[commands(food, week, image, foodstats, best, worst, nextweek)]
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
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("info,sqlx::query=error"))
        .init();

    let database = Database::new().await;

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut admins = Vec::new();
    let adminfile = File::open("admins.txt").unwrap();
    let adminfilereader = BufReader::new(adminfile);
    for line in adminfilereader.lines() {
        if let Ok(i) = line.unwrap().parse::<u64>() {
            admins.push(i)
        };
    }

    let mut image_providers = Vec::new();
    let image_providersfile = File::open("image_providers.txt").unwrap();
    let image_provider_reader = BufReader::new(image_providersfile);
    for line in image_provider_reader.lines() {
        if let Ok(i) = line.unwrap().parse::<u64>() {
            image_providers.push(i)
        };
    }

    let image_blog = match env::var("IMAGE_CHANNEL_ID") {
        Ok(i) => i.parse::<u64>().unwrap(),
        Err(_) => 0,
    };

    let roleids = RoleIDs {
        admin: admins,
        image_provider: image_providers,
        image_blog,
    };

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("!"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<RoleIDs>(roleids);
        data.insert::<Database>(Arc::new(database));
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
