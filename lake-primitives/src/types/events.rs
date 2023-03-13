use super::receipts::Receipt;

#[derive(Clone, Debug)]
pub struct Event {
    pub(crate) related_receipt_id: crate::CryptoHash,
    pub(crate) raw_event: RawEvent,
}

impl Event {
    pub fn event(&self) -> String {
        self.raw_event.event.clone()
    }

    pub fn standard(&self) -> String {
        self.raw_event.standard.clone()
    }

    pub fn version(&self) -> String {
        self.raw_event.version.clone()
    }

    pub fn data(&self) -> Option<serde_json::Value> {
        self.raw_event.data.clone()
    }

    pub fn related_receipt_id(&self) -> &crate::CryptoHash {
        &self.related_receipt_id
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RawEvent {
    pub event: String,
    pub standard: String,
    pub version: String,
    pub data: Option<serde_json::Value>,
}

impl RawEvent {
    pub fn from_log(log: &str) -> anyhow::Result<Self> {
        let prefix = "EVENT_JSON:";
        if !log.starts_with(prefix) {
            anyhow::bail!("log message doesn't start from required prefix");
        }

        Ok(serde_json::from_str::<'_, Self>(
            log[prefix.len()..].trim(),
        )?)
    }
}

pub trait EventsTrait<Receipt> {
    fn events(&self) -> Vec<Event>;
}

impl EventsTrait<Receipt> for Receipt {
    fn events(&self) -> Vec<Event> {
        self.logs()
            .iter()
            .filter_map(|log| RawEvent::from_log(log).ok())
            .map(|raw_event| Event {
                related_receipt_id: self.receipt_id(),
                raw_event,
            })
            .collect()
    }
}
