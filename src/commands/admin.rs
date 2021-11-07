use crate::commands::update_ruokadb::*;
use crate::RoleIDs;
use serenity::http::GuildPagination;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use tracing::info;

pub async fn handle_admin_message(ctx: Context, msg: Message) -> Result<(), ()> {
    if msg.content.starts_with("!update") {
        let link = match msg.content.len() {
            7 => None,
            _ => Some(msg.content[msg.content.find(' ').unwrap() + 1..].to_owned()),
        };
        match update_ruokadb(&ctx, link).await {
            Ok(_) => {
                msg.channel_id
                    .say(&ctx.http, "Ruokalista päivitetty!")
                    .await
                    .unwrap();
                return Ok(());
            }
            Err(_) => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Ruokalistan päivittäminen epäonnistui. Ehkä se on jo ajan tasalla?",
                    )
                    .await
                    .unwrap();
                return Ok(());
            }
        };
    } else if msg.content.starts_with("!broadcast") {
        if msg.content.len() == 10 {
            msg.reply(&ctx.http, "Please provide the message to be broadcasted!")
                .await
                .unwrap();
        } else {
            let image_blog = ctx.data.read().await.get::<RoleIDs>().unwrap().image_blog;
            let mainserver = ctx
                .http
                .get_channel(image_blog)
                .await
                .unwrap()
                .guild()
                .unwrap()
                .guild_id;
            let message = msg.content[msg.content.find(' ').unwrap() + 1..].to_string();
            let mut defaultids: Vec<u64> = mainserver.channels(&ctx.http).await.unwrap().into_keys().map(|x| x.0).collect();
            defaultids.sort_unstable();
            // The channer with the lowest id-number is more often than not #general
            for defid in defaultids {
                if let Channel::Guild(gc) = ctx.http.get_channel(defid).await.unwrap() {
                    match gc.id.say(&ctx.http, format!("@everyone {}", message)).await {
                        Ok(_) => {
                            info!("Broadcasting {} on the default server", message);
                            break
                        },
                        _ => continue,
                    }
                }
            }
            for guild in ctx
                .http
                .get_guilds(&GuildPagination::After(mainserver), 100)
                .await
                .unwrap()
            {
                let mut idvec: Vec<u64> = guild.id.channels(&ctx.http).await.unwrap().into_keys().map(|x| x.0).collect();
                idvec.sort_unstable();
                // The channer with the lowest id-number is more often than not #general
                for id in idvec {
                    if let Channel::Guild(gc) = ctx.http.get_channel(id).await.unwrap() {
                        match gc.id.say(&ctx.http, format!("@everyone {}", message)).await {
                            Ok(_) => {
                                info!("Broadcasting {} on {}", message, guild.name);
                                break
                            },
                            _ => continue,
                        }
                    }
                }
            }
        }
    } else if msg.content.starts_with("!lisää") {
        if msg.content.len() == 6 {
            msg.reply(
                &ctx.http,
                "The following argumenst are needed: {admin|image_provider} {userid}",
            )
            .await
            .unwrap();
        } else {
            let rooli =
                &msg.content[msg.content.find(' ').unwrap() + 1..msg.content.rfind(' ').unwrap()];
            let userid = &msg.content[msg.content.rfind(' ').unwrap() + 1..msg.content.len()]
                .parse::<u64>()
                .unwrap();
            match rooli {
                "admin" => {
                    let mut data = ctx.data.write().await;
                    let roles = &mut data.get_mut::<RoleIDs>().unwrap();
                    if roles.admin.contains(userid) {
                        msg.reply(&ctx.http, "The match is already configured!")
                            .await
                            .unwrap();
                        return Ok(());
                    }
                    roles.admin.push(*userid);
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open("admins.txt")
                        .unwrap();

                    writeln!(file, "{}", userid).unwrap();
                }
                "image_provider" => {
                    let mut data = ctx.data.write().await;
                    let roles = &mut data.get_mut::<RoleIDs>().unwrap();
                    if roles.image_provider.contains(userid) {
                        msg.reply(&ctx.http, "The match is already configured!")
                            .await
                            .unwrap();
                        return Ok(());
                    }
                    roles.image_provider.push(*userid);
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open("image_providers.txt")
                        .unwrap();

                    writeln!(file, "{}", userid).unwrap();
                }
                _ => {
                    msg.reply(&ctx.http, "Invalid role provided!")
                        .await
                        .unwrap();
                    return Ok(());
                }
            }
            msg.reply(&ctx.http, "Added! :+1:").await.unwrap();
        }
    } else if msg.content.starts_with("!poista") {
        if msg.content.len() == 7 {
            msg.reply(
                &ctx.http,
                "The following argumenst are needed: {admin|image_provider} {userid}",
            )
            .await
            .unwrap();
        } else {
            let rooli =
                &msg.content[msg.content.find(' ').unwrap() + 1..msg.content.rfind(' ').unwrap()];
            let userid = &msg.content[msg.content.rfind(' ').unwrap() + 1..msg.content.len()]
                .parse::<u64>()
                .unwrap();
            match rooli {
                "admin" => {
                    let mut data = ctx.data.write().await;
                    let roles = &mut data.get_mut::<RoleIDs>().unwrap();
                    if !roles.admin.contains(userid) {
                        msg.reply(&ctx.http, "No such match configured!")
                            .await
                            .unwrap();
                        return Ok(());
                    }
                    roles.admin.retain(|&x| x != *userid);

                    let mut adminfile = File::create("admins.txt").unwrap();
                    for admin in &roles.admin {
                        writeln!(adminfile, "{}", admin).unwrap();
                    }
                }
                "image_provider" => {
                    let mut data = ctx.data.write().await;
                    let roles = &mut data.get_mut::<RoleIDs>().unwrap();
                    if !roles.image_provider.contains(userid) {
                        msg.reply(&ctx.http, "No such match configured!")
                            .await
                            .unwrap();
                        return Ok(());
                    }
                    roles.image_provider.retain(|&x| x != *userid);

                    let mut image_providerfile = File::create("image_providers.txt").unwrap();
                    for image_provider in &roles.image_provider {
                        writeln!(image_providerfile, "{}", image_provider).unwrap();
                    }
                }
                _ => {
                    msg.reply(&ctx.http, "Invalid role provided!")
                        .await
                        .unwrap();
                    return Ok(());
                }
            }
            msg.reply(&ctx.http, "Removed! :+1:").await.unwrap();
        }
    } else if msg.content.starts_with("!listaa") {
        let data = ctx.data.read().await;
        let roles = data.get::<RoleIDs>().unwrap();
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.color(serenity::utils::Color::ORANGE);
                    e.title("Admins and image providers");
                    for admin in &roles.admin {
                        e.field("Admin", admin.to_string(), false);
                    }
                    for image_provider in &roles.image_provider {
                        e.field("Image_provider", image_provider.to_string(), false);
                    }
                    e
                })
            })
            .await
            .unwrap();
    }
    Ok(())
}
