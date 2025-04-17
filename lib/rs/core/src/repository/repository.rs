use sqlx::{Pool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;

pub trait Repository: Send + Sync {
	/// Provide access to the database connection pool.
	fn pool(&self) -> &Pool<Postgres>;

	/// Execute a function within a transaction.
	fn with_transaction<'r, F, R, E>(
		&'r self,
		execute_transaction_body: F,
	) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'r>>
	where
		F: for<'c> FnOnce(
				&'c mut Transaction<'r, Postgres>,
			) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'c>>
			+ Send
			+ 'r,
		R: Send + 'r,
		E: From<sqlx::Error> + Send + 'r,
	{
		Box::pin(async move {
			let mut tx = self.pool().begin().await?;
			let result = execute_transaction_body(&mut tx).await;

			match result {
				Ok(r) => {
					tx.commit().await?;
					Ok(r)
				}
				Err(e) => {
					let _ = tx.rollback().await;
					Err(e)
				}
			}
		})
	}
}
