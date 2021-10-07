#![allow(dead_code)]
use super::Database;

pub struct Ruoka {
    pub RuokaID: i32,
    pub RuokaName: String
}

impl Database {
    pub async fn nouda_ruoka_by_name(&self, nimi: String) -> Result<Ruoka, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka: Ruoka = sqlx::query_as!(Ruoka, "SELECT RuokaID, RuokaName FROM Ruoat WHERE RuokaName = ?", nimi)
            .fetch_one(&mut conn).await?;
        Ok(ruoka)
    }
    pub async fn nouda_ruoka_by_id(&self, id: u32) -> Result<Ruoka, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka: Ruoka = sqlx::query_as!(Ruoka, "SELECT RuokaID, RuokaName FROM Ruoat WHERE RuokaID = ?", id)
            .fetch_one(&mut conn).await?;
        Ok(ruoka)
    }
    pub async fn nouda_ruoka_by_date(&self, date: String) -> Result<String, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka = sqlx::query!("SELECT KokoRuoka FROM Ruokalista WHERE PVM = ?", date)
            .fetch_one(&mut conn).await?;
        Ok(ruoka.KokoRuoka)
    }
}
