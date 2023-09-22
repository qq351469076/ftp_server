use crate::utils::db::get_db;

pub async fn user(account: &str, password: &str) -> bool {
    let db = get_db();

    let record = sqlx::query!(
        "select account from \"user\" where account = $1 and password = $2",
        account,
        password
    )
    .fetch_optional(db)
    .await
    .unwrap();

    record.is_some()
}
