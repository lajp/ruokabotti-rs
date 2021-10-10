use super::Database;
use tracing::info;

pub struct Ruoka {
    ImageName: Option<String>
}

impl Database {
    pub async fn ruokakuva_by_name(&self, nimi:String) -> Option<String> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut ruokakuva = match sqlx::query!("SELECT ImageName FROM Ruoat WHERE RuokaName = ?", nimi)
            .fetch_one(&mut conn).await {
                Ok(r) => r.ImageName,
                Err(e) => {
                    info!("Error while querying for food image `{}`: {}", nimi, e);
                    None
                },
            };

        ruokakuva = match ruokakuva{
            Some(s) => Some(s),
            None => {
                info!("{} has non image in the database", nimi);
                None
            }
        };
        ruokakuva
    }
    pub async fn ruokakuva_by_id(&self, id:i32) -> Option<String> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut ruokakuva = match sqlx::query!("SELECT ImageName FROM Ruoat WHERE RuokaID = ?", id)
            .fetch_one(&mut conn).await {
                Ok(r) => r.ImageName,
                Err(e) => {
                    info!("Error while querying for food image ID:`{}`: {}", id, e);
                    None
                },
            };

        ruokakuva = match ruokakuva{
            Some(s) => Some(s),
            None => {
                info!("ID:{} has non image in the database", id);
                None
            }
        };
        ruokakuva
    }
    pub async fn ruokakuvat_by_query(&self, query:String) -> Option<Vec<String>> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruokakuvat = match sqlx::query!("SELECT ImageName FROM Ruoat WHERE LOWER(ImageName) LIKE LOWER( ? )", format!("%{}%", query))
            .fetch_all(&mut conn).await {
                Ok(r) => r,
                Err(e) => {
                    info!("Error while querying for food image `{}`: {}", query, e);
                    Vec::new()
                },
            };
        Some(ruokakuvat.iter().map(|r| r.ImageName.as_ref().unwrap().clone()).collect())
    }
}
