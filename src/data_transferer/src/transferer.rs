use super::*;

pub async fn executor(mut receiver: mpsc::UnboundedReceiver<tokio::task::JoinHandle<()>>) {
    loop {
        tokio::select! {
            Some(content) = receiver.recv()=> {
                content.await.expect("failed to run greenthread!");
            }
            else => break,
        }
    }
}

pub async fn transferer(
    destination_database: std::sync::Arc<Database>,
    content: Document,
    collection: String,
) {
    if let Err(result) = destination_database
        .create_by_documents(&collection, &[content], None)
        .await
    {
        println!("failed to insert! {:?}", result);
        return;
    }
}
