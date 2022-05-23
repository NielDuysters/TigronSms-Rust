/*
    Rust-module to send text-messages through Tigron's SMS-API using SOAP.
    Written by: Niel Duysters (contact@ndvibes.com)
*/

use xml::reader::{EventReader, XmlEvent};

// Client to send a text-message through Tigron's API
pub struct TigronSms {
    pub credentials: (String, String),
}

// Basic SOAP-client to interact with API
// Note: This SOAP-client will only suffice for the sms use-case.
struct SoapClient {
    pub url: String,
    pub ns: String,
    pub credentials: (String, String),
}

// Basic XML Parser to interpet the response from the Tigron-API
// Note: This parser will only suffice for interpreting the response of the 'info' procedure.
struct XmlResponseParser;

impl TigronSms {
    /*
        Method to send a text-message
        :param to: Telephone number to send message to. Format: +xx.xxxxxxxxx
        :param from: Source of message. Format: +xx.xxxxxxxxx
        :param message: Content of message to send
    */
    pub async fn send(&self, to: String, from: String, message: String) {
        let soap_client = SoapClient {
            url: "https://api.tigron.net/soap".to_string(),
            ns: "https://www.tigron.net/ns/".to_string(),
            credentials: (
                self.credentials.0.to_string(),
                self.credentials.1.to_string(),
            ),
        };

        let user_id = &*self.get_user_id().await;
        let sms_params = vec![
            ("user_id", user_id),
            ("from", &*from),
            ("to", &*to),
            ("message", &*message),
        ];

        soap_client.call("sms", "send_sms", Some(sms_params)).await;
    }

    // Function to retrieve user_id
    async fn get_user_id(&self) -> String {
        let soap_client = SoapClient {
            url: "https://api.tigron.net/soap".to_string(),
            ns: "https://www.tigron.net/ns/".to_string(),
            credentials: (
                self.credentials.0.to_string(),
                self.credentials.1.to_string(),
            ),
        };

        let response = soap_client.call("user", "info", None).await;
        let response_items = XmlResponseParser::parse(&response).await;
        let user_id = XmlResponseParser::value(&response_items, "id").await;

        user_id
    }
}

impl SoapClient {
    /*
        Send a command to the API and retrieve XML
        :param service: Service of API to execute a command on. E.g: "sms"
        :param cmd: The command to execute. E.g: "send_sms"
        :param params: Parameters of the command. E.g: [("from", "xxxx.xxx.xxx"), ("to", "yyyy.yyy.yyy")]
        :param String: Returns the body of the API-response
    */

    pub async fn call(
        &self,
        service: &str,
        cmd: &str,
        params: Option<std::vec::Vec<(&str, &str)>>,
    ) -> String {
        let http = reqwest::Client::new();

        let params = match params {
            Some(params) => params,
            None => std::vec::Vec::new(),
        };

        let cmd_xml = self.cmd_and_params_to_wsdl(cmd, params).await;
        let soap_body = self.soap_body(cmd_xml).await;

        let response = http
            .post(&format!(
                "{url}/{service}?WSDL",
                url = self.url,
                service = service
            ))
            .header("Content-Type", "application/xml")
            .body(soap_body)
            .send()
            .await
            .expect("Failed to get response.")
            .text()
            .await
            .unwrap();

        response
    }

    // Convert the array from the command and params-array into WSDL/XML format
    async fn cmd_and_params_to_wsdl(
        &self,
        cmd: &str,
        params: std::vec::Vec<(&str, &str)>,
    ) -> String {
        let mut xml = String::new();

        for param in params.iter() {
            let element = format!("<{key}>{value}</{key}>", key = param.0, value = param.1);
            xml = format!("{}{}", xml, element);
        }

        xml = format!(
            "<{cmd} xmlns=\"{ns}\">{params}</{cmd}>",
            cmd = cmd,
            params = xml,
            ns = self.ns
        );

        xml
    }

    // Function to get the full WSDL for the call
    async fn soap_body(&self, cmd_xml: String) -> String {
        let wsdl = format!(
            r#"<?xml version="1.0"?>

                <soap:Envelope
                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                soap:encodingStyle="http://www.w3.org/2003/05/soap-encoding">

                    <soap:Header>
                        <authenticate_user xmlns="{ns}">
                          <username>{username}</username>
                          <password>{password}</password>
                        </authenticate_user>
                    </soap:Header>

                    <soap:Body>
                        {cmd}
                    </soap:Body>

                </soap:Envelope>"#,
            ns = self.ns,
            username = self.credentials.0,
            password = self.credentials.1,
            cmd = cmd_xml
        );

        wsdl
    }
}

impl XmlResponseParser {
    /*
        :param xml: Takes XML as input. E.g: <item><key>xxx</key><value>yyy</value></item>
        :return Vec<(String, String)>: Returns a vector of tuples (key, value)
    */
    async fn parse(xml: &str) -> std::vec::Vec<(String, String)> {
        let mut return_items: std::vec::Vec<(String, String)> = std::vec::Vec::new();

        let parser = EventReader::from_str(xml);
        let mut start_reading = false;
        let mut read_key = false;
        let mut read_value = false;
        let mut key: String = String::new();
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    read_key = false;
                    read_value = false;

                    if name.local_name == "key" {
                        read_key = true;
                    }
                    if name.local_name == "value" {
                        read_value = true;
                    }

                    if name.local_name == "return" {
                        start_reading = true;
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    if name.local_name == "return" {
                        start_reading = false;
                    }
                }
                Ok(XmlEvent::Characters(text)) => {
                    if read_key {
                        key = text.to_string();
                    }
                    if read_value {
                        return_items.push((key.to_string(), text.to_string()));
                    }
                }
                Err(e) => {
                    break;
                }
                _ => {}
            }
        }

        return_items
    }

    /*
        Returns the value of the matching key
        :param items: Array of returned_items retrieved from API-response
        :param key: The key we want to retrieve the value from
        :return String: Returns the value matching the key
    */
    async fn value(items: &std::vec::Vec<(String, String)>, key: &str) -> String {
        for pair in items.iter() {
            if pair.0 == key {
                return pair.1.to_string();
            }
        }

        "".to_string()
    }
}
