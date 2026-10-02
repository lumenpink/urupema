/// Estado resultante do processamento de uma mensagem pelo pipeline do Urupema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    /// Mensagem processada com sucesso e despachada para o sink.
    Reinject,
    /// Mensagem descartada (ex: se configurado filtro de rejeição estrito).
    Discard,
    /// Mensagem inalterada (bypass para mensagens já processadas ou não-MIME).
    Bypass,
}
