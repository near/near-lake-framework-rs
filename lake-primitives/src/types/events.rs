use crate::AccountId;

use super::receipts::Receipt;

/// Hight-level representation of the Event according to the [Events Format](https://nomicon.io/Standards/EventsFormat.html).
/// In addition to the event this structure holds the data about the related [Receipt]: `receipt_id`, `receiver_id` and `predecessor_id`. All these fields are accessible via the corresponding getters.
#[derive(Clone, Debug)]
pub struct Event {
    pub(crate) related_receipt_id: crate::CryptoHash,
    pub(crate) receiver_id: AccountId,
    pub(crate) predecessor_id: AccountId,
    pub(crate) raw_event: RawEvent,
}

impl Event {
    /// Returns the `event` value from the [RawEvent].
    pub fn event(&self) -> &str {
        &self.raw_event.event
    }

    /// Returns the `standard` value from the [RawEvent].
    pub fn standard(&self) -> &str {
        &self.raw_event.standard
    }

    /// Returns the `version` value from the [RawEvent].
    pub fn version(&self) -> &str {
        &self.raw_event.version
    }

    /// Returns the `data` value from the [RawEvent] if present, otherwise returns `None`.
    pub fn data(&self) -> Option<&serde_json::Value> {
        self.raw_event.data.as_ref()
    }

    /// Returns the [CryptoHash](crate::CryptoHash) id of the related [Receipt].
    ///
    /// **Please note** that events are emitted through the `ExecutionOutcome` logs. In turn, the `ExecutionOutcome`
    /// is a result of the execution of the [Receipt].
    pub fn related_receipt_id(&self) -> crate::CryptoHash {
        self.related_receipt_id
    }

    /// Returns the [AccountId] of the receiver of the related [Receipt].
    pub fn related_receipt_receiver_id(&self) -> &AccountId {
        &self.receiver_id
    }

    /// Returns the [AccountId] of the predecessor of the related [Receipt].
    pub fn related_receipt_predecessor_id(&self) -> &AccountId {
        &self.predecessor_id
    }

    /// Returns true if the event is produced by the given contract id.
    pub fn is_emitted_by_contract(&self, contract_account_id: &AccountId) -> bool {
        &self.receiver_id == contract_account_id
    }
}

/// This structure is an honest representation of the Events Format standard described here
/// <https://nomicon.io/Standards/EventsFormat>
#[derive(Clone, Debug, serde::Deserialize)]
pub struct RawEvent {
    pub event: String,
    pub standard: String,
    pub version: String,
    pub data: Option<serde_json::Value>,
}

impl RawEvent {
    /// Parses the log message (originated from `ExecutionOutcome` but not limited) and returns the RawEvent.
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
    /// Reads the logs from the [Receipt] and extracts all the [Events](Event) from it into a Vec.
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
