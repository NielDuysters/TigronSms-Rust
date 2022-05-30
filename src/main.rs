mod tigron_sms;

#[tokio::main]
async fn main() {
    let tigron_sms = tigron_sms::TigronSms {
        credentials: ("".to_string(), "".to_string()),
    };

    let to = "".to_string();
    let from = "".to_string();
    let message = "test".to_string();

    tigron_sms.send(to, from, message).await;
}
