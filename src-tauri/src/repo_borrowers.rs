//! Borrower repository: CRUD, search, soft-delete.

use libsql::{params, Connection, Row};

use crate::db::DbError;
use crate::models::{Borrower, BorrowerInput};
use crate::util::{new_id, now_iso};

fn map_row(row: &Row) -> Result<Borrower, DbError> {
    Ok(Borrower {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        address: row.get(3)?,
        photo_path: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

const SELECT: &str = "SELECT id, name, phone, address, photo_path, notes, created_at, updated_at \
     FROM borrowers WHERE deleted_at IS NULL";

/// List borrowers, optionally filtered by a name/phone search term.
pub async fn list(conn: &Connection, search: Option<String>) -> Result<Vec<Borrower>, DbError> {
    let mut out = Vec::new();
    let mut rows = match search {
        Some(term) if !term.trim().is_empty() => {
            let like = format!("%{}%", term.trim());
            conn.query(
                &format!("{SELECT} AND (name LIKE ?1 OR phone LIKE ?1) ORDER BY name"),
                params![like],
            )
            .await?
        }
        _ => conn.query(&format!("{SELECT} ORDER BY name"), ()).await?,
    };
    while let Some(row) = rows.next().await? {
        out.push(map_row(&row)?);
    }
    Ok(out)
}

pub async fn get(conn: &Connection, id: &str) -> Result<Option<Borrower>, DbError> {
    let mut rows = conn
        .query(&format!("{SELECT} AND id = ?1"), params![id])
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(map_row(&row)?)),
        None => Ok(None),
    }
}

pub async fn create(conn: &Connection, input: BorrowerInput) -> Result<Borrower, DbError> {
    let id = new_id();
    let now = now_iso();
    conn.execute(
        "INSERT INTO borrowers (id, name, phone, address, photo_path, notes, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id.clone(),
            input.name,
            input.phone,
            input.address,
            input.photo_path,
            input.notes,
            now,
        ],
    )
    .await?;
    get(conn, &id).await?.ok_or(DbError::Msg("insert failed".into()))
}

pub async fn update(
    conn: &Connection,
    id: &str,
    input: BorrowerInput,
) -> Result<Borrower, DbError> {
    let now = now_iso();
    conn.execute(
        "UPDATE borrowers SET name = ?2, phone = ?3, address = ?4, photo_path = ?5, \
         notes = ?6, updated_at = ?7 WHERE id = ?1 AND deleted_at IS NULL",
        params![
            id,
            input.name,
            input.phone,
            input.address,
            input.photo_path,
            input.notes,
            now,
        ],
    )
    .await?;
    get(conn, id).await?.ok_or(DbError::Msg("borrower not found".into()))
}

/// Soft-delete: preserves history for reports.
pub async fn soft_delete(conn: &Connection, id: &str) -> Result<(), DbError> {
    let now = now_iso();
    conn.execute(
        "UPDATE borrowers SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .await?;
    Ok(())
}

/// Total outstanding principal for a borrower across their active loans.
pub async fn total_outstanding(conn: &Connection, borrower_id: &str) -> Result<f64, DbError> {
    let mut rows = conn
        .query(
            "SELECT \
               COALESCE(SUM(l.principal), 0) - COALESCE(( \
                 SELECT SUM(p.principal_component) FROM payments p \
                 JOIN loans l2 ON l2.id = p.loan_id \
                 WHERE l2.borrower_id = ?1 AND p.deleted_at IS NULL AND l2.deleted_at IS NULL \
               ), 0) \
             FROM loans l \
             WHERE l.borrower_id = ?1 AND l.deleted_at IS NULL \
               AND l.status IN ('active','overdue')",
            params![borrower_id],
        )
        .await?;
    if let Some(row) = rows.next().await? {
        Ok(row.get::<f64>(0)?)
    } else {
        Ok(0.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    async fn test_conn() -> Connection {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();
        crate::migrations::run_migrations(&conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn borrower_crud_and_search() {
        let conn = test_conn().await;

        let created = create(
            &conn,
            BorrowerInput {
                name: "Ramesh Kumar".into(),
                phone: Some("9876543210".into()),
                address: None,
                photo_path: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(created.name, "Ramesh Kumar");

        // List + search by partial name.
        let found = list(&conn, Some("rame".into())).await.unwrap();
        assert_eq!(found.len(), 1);

        // Update.
        let updated = update(
            &conn,
            &created.id,
            BorrowerInput {
                name: "Ramesh K".into(),
                phone: Some("9876543210".into()),
                address: Some("Bengaluru".into()),
                photo_path: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "Ramesh K");
        assert_eq!(updated.address.as_deref(), Some("Bengaluru"));

        // Soft delete removes from list.
        soft_delete(&conn, &created.id).await.unwrap();
        let after = list(&conn, None).await.unwrap();
        assert_eq!(after.len(), 0);
    }
}
