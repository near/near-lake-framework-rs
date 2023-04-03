use crate::AccountId;

use super::receipts::Receipt;

#[derive(Clone, Debug)]
pub struct Event {
    pub(crate) related_receipt_id: crate::CryptoHash,
    pub(crate) receiver_id: AccountId,
    pub(crate) predecessor_id: AccountId,
    pub(crate) raw_event: RawEvent,
}

impl Event {
    pub fn event(&self) -> &str {
        &self.raw_event.event
    }

    pub fn standard(&self) -> &str {
        &self.raw_event.standard
    }

    pub fn version(&self) -> &str {
        &self.raw_event.version
    }

    pub fn data(&self) -> Option<&serde_json::Value> {
        self.raw_event.data.as_ref()
    }

    pub fn related_receipt_id(&self) -> crate::CryptoHash {
        self.related_receipt_id
    }

    pub fn related_receipt_receiver_id(&self) -> &AccountId {
        &self.receiver_id
    }

    pub fn related_receipt_predecessor_id(&self) -> &AccountId {
        &self.predecessor_id
    }

    /// Returns true if the event is produced by the given contract id.
    pub fn is_emitted_by_contract(&self, contract_account_id: &AccountId) -> bool {
        &self.receiver_id == contract_account_id
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
                receiver_id: self.receiver_id(),
                predecessor_id: self.predecessor_id(),
                raw_event,
            })
            .collect()
    }
}
