use crate::domain::{EmailSink, EmailSinkError};

/// Registro de entrega armazenado em memória para asserções de teste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveredEmail {
    pub sender: String,
    pub recipient: String,
    pub payload: Vec<u8>,
}

/// Fake in-memory para o trait EmailSink, rodando em microssegundos sem dependência de processos externos.
#[derive(Debug, Default, Clone)]
pub struct InMemorySink {
    pub deliveries: Vec<DeliveredEmail>,
}

impl InMemorySink {
    pub fn new() -> Self {
        Self {
            deliveries: Vec::new(),
        }
    }

    pub fn last_delivery(&self) -> Option<&DeliveredEmail> {
        self.deliveries.last()
    }

    pub fn clear(&mut self) {
        self.deliveries.clear();
    }
}

impl EmailSink for InMemorySink {
    fn deliver(
        &mut self,
        sender: &str,
        recipient: &str,
        payload: &[u8],
    ) -> Result<(), EmailSinkError> {
        self.deliveries.push(DeliveredEmail {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            payload: payload.to_vec(),
        });
        Ok(())
    }
}
