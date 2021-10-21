use lld_leasing::context::Context;
use lld_leasing::database::Database;
use lld_leasing::{api, LldResult};

use tokio::spawn;

#[tokio::main]
async fn main() -> LldResult<()> {
    let db = Database::init()?;

    let context = Context::new();

    let api_context = context.clone();
    spawn(async {
        api::start_server(api_context).await;
    });

    context.run(db).await?;
    Ok(())
}
