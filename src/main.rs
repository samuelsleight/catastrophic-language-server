use tower_lsp::{LspService, Server};

use self::handler::Handler;

mod handler;
mod instance;

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(Handler::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
