use rusqlite::{params, Result as SqliteResult};
use crate::database::{Database, TokenUsage, TokenSummary, DailyTokenUsage};

/// Token 使用追踪器
pub struct TokenTracker {
    db: Database,
}

impl TokenTracker {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 记录 Token 使用
    pub fn record_usage(
        &self,
        model_id: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
        total_tokens: i32,
    ) -> SqliteResult<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.db.execute(|conn| {
            // 尝试更新现有记录
            let affected = conn.execute(
                "UPDATE token_usage 
                 SET prompt_tokens = prompt_tokens + ?1,
                     completion_tokens = completion_tokens + ?2,
                     total_tokens = total_tokens + ?3,
                     request_count = request_count + 1
                 WHERE model_id = ?4 AND date = ?5",
                params![prompt_tokens, completion_tokens, total_tokens, model_id, today],
            )?;

            // 如果没有更新到记录，则插入新记录
            if affected == 0 {
                conn.execute(
                    "INSERT INTO token_usage (model_id, date, prompt_tokens, completion_tokens, total_tokens, request_count, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                    params![model_id, today, prompt_tokens, completion_tokens, total_tokens, now],
                )?;
            }

            Ok(())
        })
    }

    /// 获取总体 Token 使用汇总
    pub fn get_summary(&self, start_date: Option<&str>, end_date: Option<&str>) -> SqliteResult<TokenSummary> {
        self.db.execute(|conn| {
            let (sql, params): (&str, Vec<&str>) = match (start_date, end_date) {
                (Some(start), Some(end)) => (
                    "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), 
                            COALESCE(SUM(total_tokens), 0), COALESCE(SUM(request_count), 0)
                     FROM token_usage WHERE date >= ?1 AND date <= ?2",
                    vec![start, end],
                ),
                (Some(start), None) => (
                    "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), 
                            COALESCE(SUM(total_tokens), 0), COALESCE(SUM(request_count), 0)
                     FROM token_usage WHERE date >= ?1",
                    vec![start],
                ),
                (None, Some(end)) => (
                    "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), 
                            COALESCE(SUM(total_tokens), 0), COALESCE(SUM(request_count), 0)
                     FROM token_usage WHERE date <= ?1",
                    vec![end],
                ),
                (None, None) => (
                    "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), 
                            COALESCE(SUM(total_tokens), 0), COALESCE(SUM(request_count), 0)
                     FROM token_usage",
                    vec![],
                ),
            };

            let mut stmt = conn.prepare(sql)?;
            
            let result = if params.is_empty() {
                stmt.query_row([], |row| {
                    Ok(TokenSummary {
                        total_prompt_tokens: row.get(0)?,
                        total_completion_tokens: row.get(1)?,
                        total_tokens: row.get(2)?,
                        total_requests: row.get(3)?,
                    })
                })
            } else if params.len() == 1 {
                stmt.query_row([params[0]], |row| {
                    Ok(TokenSummary {
                        total_prompt_tokens: row.get(0)?,
                        total_completion_tokens: row.get(1)?,
                        total_tokens: row.get(2)?,
                        total_requests: row.get(3)?,
                    })
                })
            } else {
                stmt.query_row([params[0], params[1]], |row| {
                    Ok(TokenSummary {
                        total_prompt_tokens: row.get(0)?,
                        total_completion_tokens: row.get(1)?,
                        total_tokens: row.get(2)?,
                        total_requests: row.get(3)?,
                    })
                })
            };

            result
        })
    }

    /// 按模型获取 Token 使用记录
    pub fn get_by_model(&self, model_id: &str, limit: i32) -> SqliteResult<Vec<TokenUsage>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, model_id, date, prompt_tokens, completion_tokens, total_tokens, request_count, created_at
                 FROM token_usage 
                 WHERE model_id = ?1 
                 ORDER BY date DESC 
                 LIMIT ?2",
            )?;

            let rows = stmt.query_map(params![model_id, limit], |row| {
                Ok(TokenUsage {
                    id: row.get(0)?,
                    model_id: row.get(1)?,
                    date: row.get(2)?,
                    prompt_tokens: row.get(3)?,
                    completion_tokens: row.get(4)?,
                    total_tokens: row.get(5)?,
                    request_count: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;

            rows.collect()
        })
    }

    /// 获取每日 Token 使用趋势
    pub fn get_daily_trend(&self, days: i32) -> SqliteResult<Vec<DailyTokenUsage>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT date, SUM(prompt_tokens), SUM(completion_tokens), SUM(total_tokens), SUM(request_count)
                 FROM token_usage 
                 GROUP BY date 
                 ORDER BY date DESC 
                 LIMIT ?1",
            )?;

            let rows = stmt.query_map(params![days], |row| {
                Ok(DailyTokenUsage {
                    date: row.get(0)?,
                    prompt_tokens: row.get(1)?,
                    completion_tokens: row.get(2)?,
                    total_tokens: row.get(3)?,
                    request_count: row.get(4)?,
                })
            })?;

            let mut results: Vec<DailyTokenUsage> = rows.collect::<SqliteResult<Vec<_>>>()?;
            results.reverse(); // 按日期正序排列
            Ok(results)
        })
    }

    /// 获取所有模型的 Token 使用汇总
    pub fn get_summary_by_models(&self) -> SqliteResult<Vec<(String, TokenSummary)>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT model_id, SUM(prompt_tokens), SUM(completion_tokens), SUM(total_tokens), SUM(request_count)
                 FROM token_usage 
                 GROUP BY model_id 
                 ORDER BY SUM(total_tokens) DESC",
            )?;

            let rows = stmt.query_map([], |row| {
                let model_id: String = row.get(0)?;
                let summary = TokenSummary {
                    total_prompt_tokens: row.get(1)?,
                    total_completion_tokens: row.get(2)?,
                    total_tokens: row.get(3)?,
                    total_requests: row.get(4)?,
                };
                Ok((model_id, summary))
            })?;

            rows.collect()
        })
    }
}
