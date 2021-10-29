use lld_leasing::context::Context;
use lld_leasing::database::Database;
use lld_leasing::{http_api, tcp_api, LldResult};

use tokio::spawn;

#[tokio::main]
async fn main() -> LldResult<()> {
    let db = Database::init()?;

    let context = Context::new();

    let http_api_context = context.clone();
    spawn(async {
        http_api::start_server(http_api_context).await;
    });

    let tcp_api_context = context.clone();
    spawn(async {
        tcp_api::start_server(tcp_api_context).await;
    });

    context.run(db).await?;
    Ok(())
}
