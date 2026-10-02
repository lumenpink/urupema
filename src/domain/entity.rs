/// Representa o payload bruto do e-mail recebido via stdin no formato RFC 5322.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEmail {
    pub bytes: Vec<u8>,
}

impl RawEmail {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Partes de texto dissecadas e higienizadas de uma mensagem.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessedParts {
    /// O corpo de texto puro original (se presente na mensagem original).
    pub original_plain: Option<String>,
    /// O corpo HTML original (se presente).
    pub original_html: Option<String>,
    /// O Markdown gerado a partir do HTML higienizado ("txt auto").
    pub markdown_auto: Option<String>,
    /// O HTML cirurgicamente higienizado (sem scripts, trackers ou folhas de estilo externas).
    pub sanitized_html: Option<String>,
}

/// Mensagem processada e pronta para reinjeção no Postfix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedEmail {
    /// O payload completo RFC 5322 reempacotado.
    pub payload: Vec<u8>,
    /// Resumo das partes processadas.
    pub parts: ProcessedParts,
}
