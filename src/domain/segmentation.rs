/// Tipos de segmentos extraídos da dissecação de um e-mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// Metadados resumidos (De, Para, Assunto, Data).
    Header,
    /// Chunks do corpo principal da mensagem.
    Body,
    /// Fecho de despedida e assinatura do remetente.
    Signature,
    /// Histórico citado de respostas anteriores.
    QuotedThread,
}

/// Representa um segmento semântico contendo tipo, conteúdo textual e índice de ordem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailSegment {
    pub kind: SegmentKind,
    pub content: String,
    pub index: usize,
}

/// Metadados de cabeçalho essenciais para busca e contextualização.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderMeta {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
}

/// Configurações para a máquina de segmentação e fatiamento semântico.
#[derive(Debug, Clone)]
pub struct SegmentationConfig {
    /// Número máximo de chunks para o corpo (padrão: 5).
    pub max_body_chunks: usize,
    /// Limite aproximado de tokens/palavras por chunk (padrão: 350).
    pub target_chunk_tokens: usize,
}

impl Default for SegmentationConfig {
    fn default() -> Self {
        Self {
            max_body_chunks: 5,
            target_chunk_tokens: 350,
        }
    }
}

/// Disseca um e-mail em blocos semânticos tipados (Header, Body chunks, Signature, QuotedThread).
pub fn segment_email(
    headers: Option<&HeaderMeta>,
    raw_text: &str,
    config: &SegmentationConfig,
) -> Vec<EmailSegment> {
    let mut segments = Vec::new();
    let mut current_index = 0;

    // 1. Header Segment
    if let Some(h) = headers {
        let mut header_lines = Vec::new();
        if let Some(ref from) = h.from {
            header_lines.push(format!("De: {from}"));
        }
        if let Some(ref to) = h.to {
            header_lines.push(format!("Para: {to}"));
        }
        if let Some(ref subject) = h.subject {
            header_lines.push(format!("Assunto: {subject}"));
        }
        if let Some(ref date) = h.date {
            header_lines.push(format!("Data: {date}"));
        }

        if !header_lines.is_empty() {
            segments.push(EmailSegment {
                kind: SegmentKind::Header,
                content: header_lines.join("\n"),
                index: current_index,
            });
            current_index += 1;
        }
    }

    // Normaliza quebras de linha
    let normalized = raw_text.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    if lines.is_empty() {
        return segments;
    }

    // 2. Localizar início de QuotedThread
    let quote_start_idx = lines.iter().position(|line| is_quote_start(line));

    let (non_quote_lines, quote_lines) = match quote_start_idx {
        Some(idx) => (&lines[..idx], Some(&lines[idx..])),
        None => (&lines[..], None),
    };

    // 3. Localizar início de Signature no conteúdo não-citado
    let sig_start_idx = non_quote_lines
        .iter()
        .position(|line| is_signature_start(line));

    let (body_lines, sig_lines) = match sig_start_idx {
        Some(idx) => (&non_quote_lines[..idx], Some(&non_quote_lines[idx..])),
        None => (non_quote_lines, None),
    };

    // 4. Fatiar o Body com estratégia Head-Tail Chunking
    let body_text = body_lines.join("\n").trim().to_string();
    if !body_text.is_empty() {
        let body_chunks = chunk_body_head_tail(&body_text, config);
        for chunk in body_chunks {
            segments.push(EmailSegment {
                kind: SegmentKind::Body,
                content: chunk,
                index: current_index,
            });
            current_index += 1;
        }
    }

    // 5. Signature Segment
    if let Some(sig) = sig_lines {
        let sig_text = sig.join("\n").trim().to_string();
        if !sig_text.is_empty() {
            segments.push(EmailSegment {
                kind: SegmentKind::Signature,
                content: sig_text,
                index: current_index,
            });
            current_index += 1;
        }
    }

    // 6. QuotedThread Segment
    if let Some(quote) = quote_lines {
        let quote_text = quote.join("\n").trim().to_string();
        if !quote_text.is_empty() {
            segments.push(EmailSegment {
                kind: SegmentKind::QuotedThread,
                content: quote_text,
                index: current_index,
            });
        }
    }

    segments
}

/// Verifica se a linha indica início de citação / histórico de resposta anterior.
fn is_quote_start(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with('>') {
        return true;
    }

    let lower = trimmed.to_lowercase();
    if lower.starts_with("---") || lower.starts_with("-----") {
        if lower.contains("original")
            || lower.contains("encaminhada")
            || lower.contains("forwarded")
            || lower.contains("reenviado")
        {
            return true;
        }
    }

    // Português: "Em ..., ... escreveu:" / "No dia ..., ... escreveu:"
    if (lower.starts_with("em ") || lower.starts_with("no dia "))
        && (lower.ends_with("escreveu:") || lower.ends_with("escreveu :"))
    {
        return true;
    }

    // Inglês: "On ..., ... wrote:"
    if lower.starts_with("on ") && (lower.ends_with("wrote:") || lower.ends_with("wrote :")) {
        return true;
    }

    // Espanhol: "El ..., ... escribió:" / "En ..., ... escribió:"
    if (lower.starts_with("el ") || lower.starts_with("en "))
        && (lower.ends_with("escribió:")
            || lower.ends_with("escribió :")
            || lower.ends_with("escribio:")
            || lower.ends_with("escribio :"))
    {
        return true;
    }

    false
}

/// Verifica se a linha indica início de fecho ou assinatura.
fn is_signature_start(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed == "--" || line.starts_with("-- ") {
        return true;
    }

    let stripped = trimmed
        .trim_end_matches(&[',', '.', ':', ';', '!', '…'][..])
        .trim();
    let lower = stripped.to_lowercase();

    const VALEDICTIONS: &[&str] = &[
        // Português (pt)
        "atenciosamente",
        "att",
        "atte",
        "cordialmente",
        "cordiais saudações",
        "cordiais saudacoes",
        "abraços",
        "abracos",
        "um abraço",
        "um abraco",
        "forte abraço",
        "forte abraco",
        "saudações",
        "saudacoes",
        "cumprimentos",
        "com os melhores cumprimentos",
        "grato",
        "obrigado",
        "obrigada",
        "desde já agradeço",
        "desde ja agradeco",
        // Inglês (en)
        "best regards",
        "warm regards",
        "regards",
        "with regards",
        "sincerely",
        "yours sincerely",
        "yours faithfully",
        "yours truly",
        "cheers",
        "best",
        "all the best",
        "thanks",
        "thank you",
        "many thanks",
        "thank you in advance",
        "respectfully",
        // Espanhol (es)
        "atentamente",
        "saludos cordiales",
        "saludos",
        "un saludo",
        "muchos saludos",
        "un abrazo",
        "fuertes abrazos",
        "gracias",
        "muchas gracias",
        "desde ya muchas gracias",
        "agradecido",
        "agradecida",
        "quedo a su disposición",
        "quedo a su disposicion",
        "quedo atento",
        "quedo atenta",
    ];

    VALEDICTIONS.contains(&lower.as_str())
}

/// Fatiamento de corpo com estratégia Cabeça-Cauda (Head-Tail Chunking).
fn chunk_body_head_tail(body: &str, config: &SegmentationConfig) -> Vec<String> {
    let paragraphs: Vec<&str> = body
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        return Vec::new();
    }

    // Agrupa parágrafos contíguos até atingir o limite aproximado de tokens
    let mut initial_chunks = Vec::new();
    let mut current_group = Vec::new();
    let mut current_tokens = 0;

    for para in paragraphs {
        let para_tokens = para.split_whitespace().count();
        if current_tokens > 0 && current_tokens + para_tokens > config.target_chunk_tokens {
            initial_chunks.push(current_group.join("\n\n"));
            current_group.clear();
            current_tokens = 0;
        }
        current_group.push(para);
        current_tokens += para_tokens;
    }

    if !current_group.is_empty() {
        initial_chunks.push(current_group.join("\n\n"));
    }

    // Se couber no teto de chunks, retorna tudo
    if initial_chunks.len() <= config.max_body_chunks {
        return initial_chunks;
    }

    // Estratégia Cabeça-Cauda: preserva 4 blocos da cabeça e 1 da cauda
    let head_count = config.max_body_chunks.saturating_sub(1);
    let mut selected_chunks = Vec::with_capacity(config.max_body_chunks);

    for chunk in initial_chunks.iter().take(head_count) {
        selected_chunks.push(chunk.clone());
    }

    // O último bloco é extraído de trás pra frente
    if let Some(tail_chunk) = initial_chunks.last() {
        selected_chunks.push(tail_chunk.clone());
    }

    selected_chunks
}
