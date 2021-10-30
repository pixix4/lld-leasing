#[macro_use]
extern crate log;

use lld_leasing::context::LldContext;
use lld_leasing::database::Database;
use lld_leasing::{http_api, tcp_api, LldResult};

use tokio::spawn;

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    {
        info!("Initialze database");
        let db = Database::open()?;
        db.init()?;
    }

    let context = LldContext::new();

    info!("Start http endpoint");
    let http_api_context = context.clone();
    spawn(async {
        http_api::start_server(http_api_context).await;
    });

    info!("Start tcp endpoint");
    let tcp_api_context = context.clone();
    spawn(async {
        tcp_api::start_server(tcp_api_context).await;
    });

    info!("Start working queue");
    context.run().await?;
    Ok(())
}
