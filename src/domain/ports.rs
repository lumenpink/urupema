use thiserror::Error;

/// Erros decorrentes da entrega via sink (ex: sendmail pipe).
#[derive(Debug, Error)]
pub enum EmailSinkError {
    #[error("Erro de I/O na execução do sink: {0}")]
    Io(#[from] std::io::Error),
    #[error("O comando do sink encerrou com status de erro: {0}")]
    CommandFailed(i32),
}

/// Porta de saída (Output Port) para entrega da mensagem processada.
pub trait EmailSink {
    /// Despacha o payload RFC 5322 para o destino final (ex: sendmail subprocess ou in-memory buffer).
    fn deliver(
        &mut self,
        sender: &str,
        recipient: &str,
        payload: &[u8],
    ) -> Result<(), EmailSinkError>;
}
