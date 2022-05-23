mod tigron_sms;

#[tokio::main]
async fn main() {
    let tigron_sms = tigron_sms::TigronSms {
        credentials: ("nieltest".to_string(), "parfaz-basziv-daqcE6".to_string()),
    };

    let to = "+32.479374727".to_string();
    let from = "+32.479374727".to_string();
    let message = "test".to_string();

    tigron_sms.send(to, from, message).await;
}
