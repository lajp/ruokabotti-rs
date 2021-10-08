use super::Database;
use tracing::info;

impl Database {
    pub async fn ruokakuva_by_name(&self, nimi:String) -> Option<String> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut ruokakuva = match sqlx::query!("SELECT ImageName FROM Ruoat WHERE RuokaName = ?", nimi)
            .fetch_one(&mut conn).await {
                Ok(r) => r.ImageName,
                Err(e) => {
                    info!("Error while querying for food image {}: {}", nimi, e);
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
}
