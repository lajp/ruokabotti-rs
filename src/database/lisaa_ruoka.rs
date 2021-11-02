#![allow(dead_code)]
use super::Database;
use chrono::NaiveDate;
use tracing::info;

impl Database {
    pub async fn lisaa_kuva_ruokaan(
        &self,
        id: i32,
        kuva: String,
    ) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!("UPDATE Ruoat SET ImageName = ? WHERE RuokaID = ?", kuva, id)
            .execute(&mut conn)
            .await
    }
    pub async fn lisaa_uusi_ruoka(&self, nimi: String) -> Result<i32, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let res = sqlx::query!("INSERT INTO Ruoat (RuokaName) VALUES ( ? )", nimi)
            .execute(&mut conn)
            .await?;
        match res.rows_affected() {
            r if r > 0 => Ok(self.nouda_ruoka_by_name(nimi).await.unwrap().RuokaID),
            _ => Err(sqlx::error::Error::RowNotFound),
        }
    }
    pub async fn lisaa_ruoat_listaan(&self, lista: Vec<(NaiveDate, String)>) -> Result<(), ()> {
        let mut conn = self.pool.acquire().await.unwrap();
        for item in lista {
            info!("Item: {:?}", item);
            let ruokaname = &item.1[..item.1.find(",").unwrap()].to_string();
            let foodid = match self.nouda_ruoka_by_name(ruokaname.to_string()).await {
                Ok(r) => r.RuokaID,
                Err(_) => self.lisaa_uusi_ruoka(ruokaname.to_string()).await.unwrap(),
            };
            let res = sqlx::query!(
                "INSERT INTO Ruokalista (PVM, RuokaID, KokoRuoka) VALUES( ? , ? , ?)",
                &item.0.to_string(),
                &foodid,
                &item.1
            )
            .execute(&mut conn)
            .await;
            match res {
                Ok(_) => continue,
                Err(_) => return Err(()),
            }
        }
        Ok(())
    }
}
