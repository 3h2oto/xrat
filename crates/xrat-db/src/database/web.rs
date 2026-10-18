use super::Database;
use crate::connection::DbPool;
use sqlx::Row;

#[derive(Clone, Debug)]
pub struct WebUser {
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub enabled: bool,
    pub must_change_password: bool,
    pub credential_version: String,
}
#[derive(Clone, Debug)]
pub struct WebSession {
    pub token_hash: String,
    pub username: String,
    pub credential_version: String,
    pub csrf_hash: String,
    pub created_at: i64,
    pub accessed_at: i64,
}
#[derive(Clone, Debug)]
pub struct WebToken {
    pub token_hash: String,
    pub username: String,
    pub credential_version: String,
    pub scope: String,
    pub name: String,
    pub created_at: i64,
}
#[derive(Clone, Debug)]
pub struct WebJob {
    pub id: String,
    pub username: String,
    pub kind: String,
    pub state: String,
    pub result: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

macro_rules! execute {
    ($self:ident, $sql:expr $(, $value:expr)* $(,)?) => {
        match &$self.pool {
            DbPool::Sqlite(pool) => sqlx::query($sql)$(.bind($value))*.execute(pool).await?.rows_affected(),
            DbPool::Postgres(pool) => sqlx::query($sql)$(.bind($value))*.execute(pool).await?.rows_affected(),
        }
    };
}
macro_rules! fetch {
    ($self:ident, $sql:expr, $map:expr $(, $value:expr)* $(,)?) => {
        match &$self.pool {
            DbPool::Sqlite(pool) => sqlx::query($sql)$(.bind($value))*.fetch_all(pool).await?.into_iter().map($map).collect(),
            DbPool::Postgres(pool) => sqlx::query($sql)$(.bind($value))*.fetch_all(pool).await?.into_iter().map($map).collect(),
        }
    };
}
impl Database {
    pub async fn web_audit(&self, actor: &str, action: &str) -> crate::Result<()> {
        execute!(
            self,
            "INSERT INTO events(level,source,kind,message,actor) VALUES('info','api','management',$1,$2)",
            action,
            actor
        );
        Ok(())
    }

    pub async fn web_users(&self) -> crate::Result<Vec<WebUser>> {
        Ok(fetch!(
            self,
            "SELECT * FROM web_users ORDER BY username",
            |row| WebUser {
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                enabled: row.get::<i64, _>("enabled") != 0,
                must_change_password: row.get::<i64, _>("must_change_password") != 0,
                credential_version: row.get("credential_version"),
            }
        ))
    }
    pub async fn web_save_user(&self, user: &WebUser) -> crate::Result<()> {
        execute!(
            self,
            "INSERT INTO web_users (username,password_hash,role,enabled,must_change_password,credential_version) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT(username) DO UPDATE SET password_hash=excluded.password_hash,role=excluded.role,enabled=excluded.enabled,must_change_password=excluded.must_change_password,credential_version=excluded.credential_version",
            &user.username,
            &user.password_hash,
            &user.role,
            i64::from(user.enabled),
            i64::from(user.must_change_password),
            &user.credential_version
        );
        Ok(())
    }
    pub async fn web_save_session(&self, session: &WebSession) -> crate::Result<()> {
        execute!(
            self,
            "INSERT INTO web_sessions (token_hash,username,credential_version,csrf_hash,created_at,accessed_at) VALUES ($1,$2,$3,$4,$5,$6)",
            &session.token_hash,
            &session.username,
            &session.credential_version,
            &session.csrf_hash,
            session.created_at,
            session.accessed_at
        );
        Ok(())
    }
    pub async fn web_session(&self, token_hash: &str) -> crate::Result<Option<WebSession>> {
        let rows: Vec<WebSession> = fetch!(
            self,
            "SELECT * FROM web_sessions WHERE token_hash=$1",
            |row| WebSession {
                token_hash: row.get("token_hash"),
                username: row.get("username"),
                credential_version: row.get("credential_version"),
                csrf_hash: row.get("csrf_hash"),
                created_at: row.get("created_at"),
                accessed_at: row.get("accessed_at"),
            },
            token_hash
        );
        Ok(rows.into_iter().next())
    }
    pub async fn web_touch_session(&self, hash: &str, now: i64) -> crate::Result<()> {
        execute!(
            self,
            "UPDATE web_sessions SET accessed_at=$1 WHERE token_hash=$2",
            now,
            hash
        );
        Ok(())
    }
    pub async fn web_delete_session(&self, hash: &str) -> crate::Result<()> {
        execute!(self, "DELETE FROM web_sessions WHERE token_hash=$1", hash);
        Ok(())
    }
    pub async fn web_revoke_user(&self, username: &str) -> crate::Result<()> {
        execute!(self, "DELETE FROM web_sessions WHERE username=$1", username);
        execute!(self, "DELETE FROM web_tokens WHERE username=$1", username);
        Ok(())
    }
    pub async fn web_prune_sessions(&self, now: i64) -> crate::Result<()> {
        execute!(
            self,
            "DELETE FROM web_sessions WHERE accessed_at < $1 OR created_at < $2",
            now - 43200,
            now - 604800
        );
        Ok(())
    }
    pub async fn web_tokens(&self, username: &str) -> crate::Result<Vec<WebToken>> {
        Ok(fetch!(
            self,
            "SELECT * FROM web_tokens WHERE username=$1 ORDER BY created_at DESC",
            |row| WebToken {
                token_hash: row.get("token_hash"),
                username: row.get("username"),
                credential_version: row.get("credential_version"),
                scope: row.get("scope"),
                name: row.get("name"),
                created_at: row.get("created_at"),
            },
            username
        ))
    }
    pub async fn web_token(&self, hash: &str) -> crate::Result<Option<WebToken>> {
        let rows: Vec<WebToken> = fetch!(
            self,
            "SELECT * FROM web_tokens WHERE token_hash=$1",
            |row| WebToken {
                token_hash: row.get("token_hash"),
                username: row.get("username"),
                credential_version: row.get("credential_version"),
                scope: row.get("scope"),
                name: row.get("name"),
                created_at: row.get("created_at"),
            },
            hash
        );
        Ok(rows.into_iter().next())
    }
    pub async fn web_save_token(&self, token: &WebToken) -> crate::Result<()> {
        execute!(
            self,
            "INSERT INTO web_tokens(token_hash,username,credential_version,scope,name,created_at) VALUES($1,$2,$3,$4,$5,$6)",
            &token.token_hash,
            &token.username,
            &token.credential_version,
            &token.scope,
            &token.name,
            token.created_at
        );
        Ok(())
    }
    pub async fn web_delete_token(&self, hash: &str, username: &str) -> crate::Result<()> {
        execute!(
            self,
            "DELETE FROM web_tokens WHERE token_hash=$1 AND username=$2",
            hash,
            username
        );
        Ok(())
    }
    pub async fn web_jobs(&self) -> crate::Result<Vec<WebJob>> {
        Ok(fetch!(
            self,
            "SELECT * FROM web_jobs ORDER BY created_at DESC LIMIT 200",
            |row| WebJob {
                id: row.get("id"),
                username: row.get("username"),
                kind: row.get("kind"),
                state: row.get("state"),
                result: row.get("result"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        ))
    }
    pub async fn web_save_job(&self, job: &WebJob) -> crate::Result<()> {
        execute!(
            self,
            "INSERT INTO web_jobs(id,username,kind,state,result,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO UPDATE SET state=excluded.state,result=excluded.result,updated_at=excluded.updated_at",
            &job.id,
            &job.username,
            &job.kind,
            &job.state,
            &job.result,
            job.created_at,
            job.updated_at
        );
        Ok(())
    }
    pub async fn web_job_progress(&self, id: &str, message: &str, now: i64) -> crate::Result<()> {
        execute!(
            self,
            "UPDATE web_jobs SET result=$1,updated_at=$2 WHERE id=$3 AND state='running'",
            message,
            now,
            id
        );
        Ok(())
    }
    pub async fn web_interrupt_jobs(&self, now: i64) -> crate::Result<()> {
        execute!(
            self,
            "UPDATE web_jobs SET state='interrupted',updated_at=$1 WHERE state IN ('queued','running')",
            now
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "web_tests.rs"]
mod tests;
