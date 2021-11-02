use crate::database::*;
use crate::util::parse_lykeion::*;
use serenity::prelude::*;
use tracing::info;

pub async fn update_ruokadb(ctx: &Context, link: Option<String>) -> Result<(), ()> {
    let lista = parse_lykeion(link).await.unwrap();
    info!("Parsed {} foods from the pdf", lista.len());
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    match db.lisaa_ruoat_listaan(lista).await {
        Err(_) => Err(()),
        _ => Ok(()),
    }
}
