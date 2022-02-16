use jack::Client;

pub struct JackDeviceEnumerator {
    dummy_client: Option<jack::Client>,
}
