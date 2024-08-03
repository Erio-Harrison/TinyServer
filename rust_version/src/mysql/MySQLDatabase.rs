use mysql::*;
use mysql::prelude::*;
use std::sync::{Arc, Mutex};

struct MySQLConnection {
    conn: PooledConn,
}

impl MySQLConnection {
    fn new(pool: &Arc<Mutex<Pool>>) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = pool.lock()?.get_conn()?;
        Ok(MySQLConnection { conn })
    }

    fn execute_query(&mut self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.query_drop(query)?;
        Ok(())
    }
}

struct MySQLDatabaseManager {
    pool: Arc<Mutex<Pool>>,
}

impl MySQLDatabaseManager {
    fn new(url: &str, max_connections: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = Pool::new(url)?;
        let pool = Arc::new(Mutex::new(pool));
        Ok(MySQLDatabaseManager { pool })
    }

    fn get_connection(&self) -> Result<MySQLConnection, Box<dyn std::error::Error>> {
        MySQLConnection::new(&self.pool)
    }

    fn execute_query(&self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection()?;
        conn.execute_query(query)
    }
}
