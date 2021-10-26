#![allow(dead_code, non_snake_case)]
use super::Database;
use tracing::info;

pub struct Ruoka {
    pub RuokaID: i32,
    pub RuokaName: String
}
pub struct RuokaJaId {
    pub RuokaID: i32,
    pub KokoRuoka: String
}
pub struct RuokalistaEntry {
    KokoRuoka: String
}

impl Database {
    pub async fn nouda_ruoka_by_name(&self, nimi: String) -> Result<Ruoka, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka: Ruoka = sqlx::query_as!(Ruoka, "SELECT RuokaID, RuokaName FROM Ruoat WHERE RuokaName = ?", nimi)
            .fetch_one(&mut conn).await?;
        Ok(ruoka)
    }
    pub async fn nouda_ruoka_by_name_case_insensitive(&self, nimi: String) -> Result<Ruoka, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka: Ruoka = sqlx::query_as!(Ruoka, "SELECT RuokaID, RuokaName FROM Ruoat WHERE LOWER(RuokaName) = LOWER(?)", nimi)
            .fetch_one(&mut conn).await?;
        Ok(ruoka)
    }
    pub async fn nouda_ruoka_by_id(&self, id: u32) -> Result<Ruoka, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoka: Ruoka = sqlx::query_as!(Ruoka, "SELECT RuokaID, RuokaName FROM Ruoat WHERE RuokaID = ?", id)
            .fetch_one(&mut conn).await?;
        Ok(ruoka)
    }
    pub async fn nouda_ruoka_by_date(&self, date: String) -> Result<Option<String>, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query!("SELECT KokoRuoka FROM Ruokalista WHERE PVM = ?", date)
            .fetch_one(&mut conn).await {
                Ok(r) => Ok(Some(r.KokoRuoka)),
                Err(_) => Ok(None)
            }
    }
    pub async fn nouda_ruoka_ja_id_by_date(&self, date: String) -> Result<Option<RuokaJaId>, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query_as!(RuokaJaId, "SELECT RuokaID, KokoRuoka FROM Ruokalista WHERE PVM = ?", date)
            .fetch_one(&mut conn).await {
                Ok(r) => Ok(Some(r)),
                Err(_) => Ok(None)
            }
    }
    pub async fn nouda_viikko(&self, alku:String, loppu:String) -> Option<Vec<String>> {
        let mut conn = self.pool.acquire().await.unwrap();
        let viikko = match sqlx::query_as!(RuokalistaEntry, "SELECT KokoRuoka FROM Ruokalista WHERE PVM BETWEEN ? AND ?", alku, loppu)
            .fetch_all(&mut conn).await {
                Ok(r) => r,
                Err(e) => {
                    info!("Error while querying for foods between `{}` and `{}`: {}", alku, loppu, e);
                    return None;
                },
            };
        Some(viikko.iter().map(|r| r.KokoRuoka.clone()).collect())
    }
}
