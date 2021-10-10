use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::info;
use tokio::fs::File;
use tokio::io;
use crate::database::*;

pub async fn handle_image_provider_message(ctx: Context, msg: Message) -> Result<(), ()> {
    if !msg.attachments.is_empty() {
        info!("The message has the following attachments {}", msg.attachments.iter().map(|a| a.filename.to_string()).collect::<Vec<String>>().join(", "));
        if msg.content == "" {
            msg.reply(&ctx.http, "Jotta ruokabotti voisi lisätä kuvan tietokantaan, on sille annettava ruoan nimi.").await.unwrap();
        }
        else {
            let db = ctx.data.read().await.get::<Database>().unwrap().clone();
            let ruoka = match db.nouda_ruoka_by_name_case_insensitive(msg.content.clone()).await {
                Ok(r) => r,
                Err(_) => {
                    msg.reply(&ctx.http, "Kyseistä ruokaa ei ole vielä määritelty tietokantaan! Tarkista, että kirjoitit nimen oikein.").await.unwrap();
                    return Ok(())
                }
            };
            let r_client = reqwest::Client::builder().build().unwrap();
            let r_res = r_client
                .get(&msg.attachments[0].url)
                .send()
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            let mut content = r_res.as_ref();

            let kuvapath = "ruoat/";

            tokio::fs::create_dir_all(&kuvapath).await.unwrap();

            let (_, ext) = &msg.attachments[0].filename.split_at(msg.attachments[0].filename.rfind(".").unwrap());
            let filepath = format!("{}{}{}", kuvapath, msg.content, ext);
            let filename = format!("{}{}", msg.content, ext);
            let mut f = File::create(&filepath).await.unwrap();
            io::copy(&mut content, &mut f).await.unwrap();
            info!("Wrote {} bytes into the file `{}`", msg.attachments[0].size, filename);
            db.lisaa_kuva_ruokaan(ruoka.RuokaID, filename.replace(" ", "%20")).await.unwrap();
            msg.reply(&ctx.http, format!(":+1: Kuva `{}` ladattiin ja lisättiin tietokantaan!", filename)).await.unwrap();
        }
    }
    Ok(())
}
