use mail_parser::{HeaderValue, Message, MessageParser};
use std::collections::HashSet;
use thiserror::Error;

use crate::domain::{
    EmailSink, EmailSinkError, ProcessResult, ProcessedParts, RawEmail, SanitizedEmail,
};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Erro ao analisar mensagem RFC 5322: {0}")]
    ParseError(String),
    #[error("Erro de entrega no sink: {0}")]
    SinkError(#[from] EmailSinkError),
    #[error("Mensagem rejeitada pela política: {0}")]
    PolicyRejected(String),
}

/// Orquestrador do pipeline de higienização e entrega MIME do Urupema.
pub struct Pipeline;

const URUPEMA_HEADER: &str = "X-Urupema-Processed: 1; engine=urupema-0.1.0\r\n";
const MIME_BOUNDARY: &str = "===============urupema_boundary==";

impl Pipeline {
    /// Processa o e-mail bruto recebido e entrega via sink.
    pub fn process<S: EmailSink>(
        raw: &RawEmail,
        sender: &str,
        recipient: &str,
        sink: &mut S,
    ) -> Result<(SanitizedEmail, ProcessResult), PipelineError> {
        let bytes = raw.as_bytes();
        let message = MessageParser::default().parse(bytes).ok_or_else(|| {
            PipelineError::ParseError("Falha ao analisar mensagem RFC 5322".to_string())
        })?;

        // 1. Prevenção contra loop no Postfix
        if Self::is_already_processed(&message) {
            sink.deliver(sender, recipient, bytes)?;
            return Ok((
                SanitizedEmail {
                    payload: bytes.to_vec(),
                    parts: ProcessedParts::default(),
                },
                ProcessResult::Bypass,
            ));
        }

        // 2. Extrair partes text/plain e text/html
        let original_plain = message.body_text(0).map(|s| s.into_owned());
        let original_html = message.body_html(0).map(|s| s.into_owned());

        let mut parts = ProcessedParts {
            original_plain,
            original_html: original_html.clone(),
            markdown_auto: None,
            sanitized_html: None,
        };

        // 3. Higienizar e converter HTML caso exista
        if let Some(ref html) = original_html {
            let mut ammonia_builder = ammonia::Builder::new();
            let allowed_tags: HashSet<&str> = [
                "p",
                "br",
                "b",
                "i",
                "strong",
                "em",
                "a",
                "ul",
                "ol",
                "li",
                "blockquote",
                "code",
                "pre",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "table",
                "thead",
                "tbody",
                "tr",
                "th",
                "td",
                "hr",
            ]
            .into_iter()
            .collect();

            ammonia_builder.tags(allowed_tags);
            ammonia_builder.link_rel(Some("noopener noreferrer"));

            let clean_html = ammonia_builder.clean(html).to_string();
            // Largura ampla para preservar fluxo livre de parágrafos sem quebras artificiais
            const UNWRAPPED_WIDTH: usize = 10_000;
            let markdown = html2text::from_read(clean_html.as_bytes(), UNWRAPPED_WIDTH);

            parts.sanitized_html = Some(clean_html);
            parts.markdown_auto = Some(markdown);
        }

        // 4. Reempacotar mensagem final
        let final_payload = Self::repackage(bytes, &parts)?;

        // 5. Entregar via sink
        sink.deliver(sender, recipient, &final_payload)?;

        Ok((
            SanitizedEmail {
                payload: final_payload,
                parts,
            },
            ProcessResult::Reinject,
        ))
    }

    fn is_already_processed(message: &Message) -> bool {
        if let Some(val) = message.header("X-Urupema-Processed") {
            match val {
                HeaderValue::Text(t) => t.contains('1'),
                HeaderValue::TextList(list) => list.iter().any(|t| t.contains('1')),
                _ => false,
            }
        } else {
            false
        }
    }

    fn repackage(original_bytes: &[u8], parts: &ProcessedParts) -> Result<Vec<u8>, PipelineError> {
        let (raw_headers, raw_body) = Self::split_headers_and_body(original_bytes);

        // Se tem HTML higienizado, construímos um multipart/alternative com markdown e HTML limpo
        if let Some(clean_html) = &parts.sanitized_html {
            let plain_content = parts
                .markdown_auto
                .as_deref()
                .or(parts.original_plain.as_deref())
                .unwrap_or("");

            let filtered_headers = Self::filter_headers(
                raw_headers,
                &["content-type", "content-transfer-encoding", "mime-version"],
            );

            let mut output = Vec::new();
            output.extend_from_slice(URUPEMA_HEADER.as_bytes());
            output.extend_from_slice(filtered_headers.as_bytes());
            output.extend_from_slice(b"MIME-Version: 1.0\r\n");
            output.extend_from_slice(
                format!(
                    "Content-Type: multipart/alternative; boundary=\"{MIME_BOUNDARY}\"\r\n\r\n"
                )
                .as_bytes(),
            );

            // Parte 1: text/plain (Markdown auto)
            output.extend_from_slice(format!("--{MIME_BOUNDARY}\r\n").as_bytes());
            output.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: 8bit\r\n\r\n");
            output.extend_from_slice(plain_content.as_bytes());
            output.extend_from_slice(b"\r\n");

            // Parte 2: text/html (Sanitized HTML)
            output.extend_from_slice(format!("--{MIME_BOUNDARY}\r\n").as_bytes());
            output.extend_from_slice(b"Content-Type: text/html; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: 8bit\r\n\r\n");
            output.extend_from_slice(clean_html.as_bytes());
            output.extend_from_slice(b"\r\n");

            // Fechamento multipart
            output.extend_from_slice(format!("--{MIME_BOUNDARY}--\r\n").as_bytes());

            Ok(output)
        } else {
            // E-mail em texto puro: preserva o corpo original e apenas injeta o cabeçalho X-Urupema
            let mut output = Vec::new();
            output.extend_from_slice(URUPEMA_HEADER.as_bytes());
            output.extend_from_slice(raw_headers);
            output.extend_from_slice(b"\r\n\r\n");
            output.extend_from_slice(raw_body);
            Ok(output)
        }
    }

    fn split_headers_and_body(bytes: &[u8]) -> (&[u8], &[u8]) {
        // Procura por \r\n\r\n
        if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            (&bytes[..pos], &bytes[pos + 4..])
        } else if let Some(pos) = bytes.windows(2).position(|w| w == b"\n\n") {
            (&bytes[..pos], &bytes[pos + 2..])
        } else {
            (bytes, b"")
        }
    }

    fn filter_headers(headers_raw: &[u8], drop_prefixes: &[&str]) -> String {
        let headers_str = String::from_utf8_lossy(headers_raw);
        let mut result = String::new();

        for line in headers_str.lines() {
            let lower = line.to_ascii_lowercase();
            let should_drop = drop_prefixes.iter().any(|prefix| lower.starts_with(prefix));
            if !should_drop && !line.trim().is_empty() {
                result.push_str(line);
                result.push_str("\r\n");
            }
        }

        result
    }
}
