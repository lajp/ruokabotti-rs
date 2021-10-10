mod commands;
mod database;
mod util;

use std::{collections::HashSet, env, sync::Arc};

use database::*;
use commands::{ruoka::*, viikko::*, kuva::*, image_provider_msg::*, admin::*};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, channel::Message},
    prelude::*,
};

use tracing::{error, info};

pub struct ShardManagerContainer;

pub struct RoleIDs {
    pub admin: u64,
    pub image_provider: u64,
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
            handle_image_provider_message(ctx, msg).await.unwrap();
        }
    }
}

#[group]
#[commands(ruoka, viikko, kuva)]
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

    let roleids = RoleIDs { admin, image_provider };

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
