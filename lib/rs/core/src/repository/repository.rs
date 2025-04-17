use sqlx::{Executor, Pool, Postgres, Transaction};
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

pub trait TransactionExt<'t> {
	/// Provide access to the inner connection for the [sqlx::Transaction].
	///
	/// As of SQLx 0.8.x, [sqlx::Transaction] doesn't implement [sqlx::Executor].
	/// However, its inner connection does, so we need to deref the transaction
	/// if we want to use it as an executor.
	fn as_executor(&'t mut self) -> impl Executor<'t, Database = Postgres> + 't;
}

impl<'t> TransactionExt<'t> for Transaction<'_, Postgres> {
	fn as_executor(&'t mut self) -> impl Executor<'t, Database = Postgres> + 't {
		// • If tx is &mut Transaction<'_, Postgres>,
		// • then *tx gives us the transaction itself, Transaction<'_, Postgres>,
		// • and **tx gives us the inner connection via the Deref trait,
		// • and &mut **tx gives us a mutable reference to that connection.
		&mut **self
	}
}
