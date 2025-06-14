use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub file_path: String,
    table_name: String,
}

pub fn create_table(conn: &Connection, config: &DatabaseConfig) -> Result<()> {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            addr INTEGER(4)
        )",
        config.table_name
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub fn get_all_addrs(conn: &Connection, config: &DatabaseConfig) -> Result<Vec<Ipv4Addr>> {
    let sql = format!("SELECT addr FROM {}", config.table_name);
    Ok(conn
        .prepare(&sql)?
        .query_map([], |row| Ok(Ipv4Addr::from(u32::from_be(row.get(0)?))))?
        .collect::<rusqlite::Result<Vec<Ipv4Addr>>>()?)
}

pub fn add_addr(conn: &Connection, addr: Ipv4Addr, config: &DatabaseConfig) -> Result<()> {
    let sql = format!(
        "INSERT OR IGNORE INTO {} (addr) VALUES (?)",
        config.table_name
    );
    conn.execute(&sql, params![addr.to_bits()])?;
    Ok(())
}

pub fn remove_addr(conn: &Connection, addr: Ipv4Addr, config: &DatabaseConfig) -> Result<()> {
    let sql = format!("DELETE FROM {} WHERE addr = ?", config.table_name);
    conn.execute(&sql, params![addr.to_bits()])?;
    Ok(())
}

pub fn check_addr(conn: &Connection, addr: Ipv4Addr, config: &DatabaseConfig) -> Result<bool> {
    let sql = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE addr = ?)",
        config.table_name
    );
    Ok(conn.query_row(&sql, params![addr.to_bits()], |row| row.get(0))?)
}
