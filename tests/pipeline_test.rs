use urupema::adapters::InMemorySink;
use urupema::application::Pipeline;
use urupema::domain::{ProcessResult, RawEmail};

#[test]
fn test_plain_text_email_preservation() {
    let raw_rfc5322 = b"From: remetente@exemplo.org\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Teste Simples\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Ola camarada! Esta e uma mensagem em texto puro.";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, result) = Pipeline::process(
        &raw,
        "remetente@exemplo.org",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Pipeline deve processar com sucesso");

    assert_eq!(result, ProcessResult::Reinject);
    assert_eq!(sink.deliveries.len(), 1);

    let delivered = sink.last_delivery().expect("Deve ter sido entregue");
    assert_eq!(delivered.sender, "remetente@exemplo.org");
    assert_eq!(delivered.recipient, "destinatario@exemplo.org");

    let payload_str = String::from_utf8_lossy(&delivered.payload);
    assert!(payload_str.contains("X-Urupema-Processed: 1"));
    assert!(payload_str.contains("Ola camarada!"));
    assert_eq!(
        sanitized.parts.original_plain.as_deref(),
        Some("Ola camarada! Esta e uma mensagem em texto puro.")
    );
}

#[test]
fn test_html_sanitization_and_markdown_conversion() {
    let raw_rfc5322 = b"From: newsletter@empresa.com\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Promocao Imperdivel\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<h1>Ola!</h1>\
<p>Visite nosso <a href=\"https://empresa.com/promo\">link promocional</a>.</p>\
<script>alert('malicioso');</script>\
<img src=\"https://tracker.com/pixel.gif\" width=\"1\" height=\"1\">\
</body></html>";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, result) = Pipeline::process(
        &raw,
        "newsletter@empresa.com",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Pipeline deve processar HTML");

    assert_eq!(result, ProcessResult::Reinject);
    assert_eq!(sink.deliveries.len(), 1);

    let delivered = sink.last_delivery().unwrap();
    let payload_str = String::from_utf8_lossy(&delivered.payload);

    // Deve conter header de rastreio
    assert!(payload_str.contains("X-Urupema-Processed: 1"));

    // Script DEVE ser removido
    assert!(!payload_str.contains("<script>"));
    assert!(!payload_str.contains("alert('malicioso')"));

    // Deve conter versão Markdown gerada (txt auto)
    assert!(sanitized.parts.markdown_auto.is_some());
    let md = sanitized.parts.markdown_auto.as_ref().unwrap();
    assert!(md.contains("Ola!"));
    assert!(md.contains("link promocional"));
    assert!(md.contains("https://empresa.com/promo"));

    // O HTML higienizado deve manter o link com rel noopener noreferrer
    let clean_html = sanitized.parts.sanitized_html.as_ref().unwrap();
    assert!(!clean_html.contains("<script>"));
    assert!(clean_html.contains("rel=\"noopener noreferrer\""));
}

#[test]
fn test_loop_prevention_on_already_processed_message() {
    let raw_rfc5322 = b"From: bot@exemplo.org\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Mensagem em Loop\r\n\
X-Urupema-Processed: 1; engine=urupema-0.1.0\r\n\
\r\n\
Mensagem ja processada.";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (_sanitized, result) = Pipeline::process(
        &raw,
        "bot@exemplo.org",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Deve identificar bypass");

    // Prevenção contra loop no Postfix
    assert_eq!(result, ProcessResult::Bypass);
    assert_eq!(sink.deliveries.len(), 1); // Pass-through inalterado
}

#[test]
fn test_line_break_preservation_and_unwrapped_paragraphs() {
    let raw_rfc5322 = b"From: remetente@exemplo.org\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Teste de Quebras e Largura\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<p>Este e um paragrafo propositalmente longo contendo bem mais de oitenta caracteres para certificar que o urupema nao impoe quebra de linha artificial arbitraria no meio das frases do texto.</p>\
<p>Endereco:<br>Rua das Flores, 123<br>Centro - Florianopolis - SC<br>CEP: 88000-000</p>\
</body></html>";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "remetente@exemplo.org",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Deve processar pipeline");

    let md = sanitized
        .parts
        .markdown_auto
        .expect("Deve gerar markdown auto");

    // 1. O paragrafo longo NAO deve conter quebras artificiais (\n) no meio
    assert!(
        md.contains("Este e um paragrafo propositalmente longo contendo bem mais de oitenta caracteres para certificar que o urupema nao impoe quebra de linha artificial arbitraria no meio das frases do texto.")
    );

    // 2. As quebras duras (<br>) do endereco devem ser preservadas como linhas distintas
    assert!(md.contains("Rua das Flores, 123\n"));
    assert!(md.contains("Centro - Florianopolis - SC\n"));
    assert!(md.contains("CEP: 88000-000"));
}

#[test]
fn test_edge_cases_and_complex_html_formatting() {
    let raw_rfc5322 = b"From: remetente@exemplo.org\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Teste de Casos Limite\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<p>Poesia:<br><br>Linha 1<br>Linha 2<br><br>Fim da estrofe.</p>\
<ul>\
<li>Primeiro item com descricao bem longa para garantir fluidez</li>\
<li>Segundo item</li>\
</ul>\
<blockquote>Citacao em bloco com multiplas palavras</blockquote>\
</body></html>";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "remetente@exemplo.org",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Deve processar pipeline");

    let md = sanitized
        .parts
        .markdown_auto
        .expect("Deve gerar markdown auto");

    // Quebras duplas preservadas
    assert!(md.contains("Linha 1\nLinha 2"));
    // Lista renderizada
    assert!(md.contains("Primeiro item com descricao bem longa para garantir fluidez"));
    assert!(md.contains("Segundo item"));
    // Blockquote
    assert!(md.contains("Citacao em bloco com multiplas palavras"));
}

#[test]
fn test_markdown_noise_sanitization_underlines_and_markers() {
    let raw_rfc5322 = b"From: newsletter@news.org\r\n\
To: destinatario@exemplo.org\r\n\
Subject: Comunicado com ruidos\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<h2>Comunicado do GitHub Universe</h2>\
<p>Destaques do evento anual.</p>\
<p>__________________________________________________</p>\
<p>Avisos importantes da comunidade.</p>\
<p>____</p>\
<p>Mais detalhes em breve.</p>\
</body></html>";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "newsletter@news.org",
        "destinatario@exemplo.org",
        &mut sink,
    )
    .expect("Deve processar pipeline");

    let md = sanitized
        .parts
        .markdown_auto
        .expect("Deve gerar markdown");

    // Nao deve conter sequencias repetitivas de underlines
    assert!(
        !md.contains("____________________"),
        "Markdown nao deve conter underlines longos repetitivos: {}",
        md
    );
    assert!(
        !md.contains("____"),
        "Markdown nao deve conter sequencias de 4+ underlines: {}",
        md
    );
    assert!(md.contains("Comunicado do GitHub Universe"));
    assert!(md.contains("Destaques do evento anual."));
    assert!(md.contains("Avisos importantes da comunidade."));
}

#[test]
fn test_payload_selection_fallback_to_plain_on_empty_or_polluted_html() {
    let raw_rfc5322 = b"From: updates@pinterest.com\r\n\
To: usuario@exemplo.org\r\n\
Subject: Novas ideias para seu painel\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/alternative; boundary=\"boundary_pin\"\r\n\
\r\n\
--boundary_pin\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Veja as novas ideias de decoracao de primavera que separamos para voce no Pinterest.\r\n\
--boundary_pin\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<table width=\"100%\"><tr><td></td></tr></table>\
<div>   </div>\
<img src=\"https://pinterest.com/pixel.gif\" width=\"1\" height=\"1\" />\
</body></html>\r\n\
--boundary_pin--";

    let raw = RawEmail::new(raw_rfc5322.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "updates@pinterest.com",
        "usuario@exemplo.org",
        &mut sink,
    )
    .expect("Deve processar e-mail multipart do Pinterest");

    let delivered = sink.last_delivery().expect("Deve ter sido entregue");
    let delivered_payload = String::from_utf8_lossy(&delivered.payload);

    // Como o HTML era vazio / sem conteudo semantico, o texto entregue no multipart
    // DEVE conter o texto do original_plain
    assert!(
        delivered_payload.contains("Veja as novas ideias de decoracao de primavera"),
        "O payload final deve conter o texto do original_plain quando HTML for vazio/poluido. Payload: {}",
        delivered_payload
    );

    // O campo markdown_auto em parts deve conter ou ter fallback para o texto útil
    let plain_chosen = sanitized
        .parts
        .markdown_auto
        .as_deref()
        .or(sanitized.parts.original_plain.as_deref())
        .unwrap_or("");
    assert!(plain_chosen.contains("Veja as novas ideias de decoracao de primavera"));
}
