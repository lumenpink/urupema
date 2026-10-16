use urupema::adapters::InMemorySink;
use urupema::application::Pipeline;
use urupema::domain::{
    HeaderMeta, RawEmail, SegmentKind, SegmentationConfig, segment_email,
};

#[test]
fn test_fixture_militar_fwd_pipeline_and_segmentation() {
    let fixture_bytes = include_bytes!("fixtures/fixture_militar_fwd.eml");
    let raw = RawEmail::new(fixture_bytes.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "servidor@exemplo.gov.br",
        "destinatario@exemplo.gov.br",
        &mut sink,
    )
    .expect("Deve processar pipeline do e-mail militar com encaminhamento");

    // Valida que Markdown Auto e HTML higienizado foram gerados
    assert!(sanitized.parts.sanitized_html.is_some());
    assert!(sanitized.parts.markdown_auto.is_some());

    let md = sanitized.parts.markdown_auto.as_ref().unwrap();

    let headers = HeaderMeta {
        from: Some("servidor@exemplo.gov.br".to_string()),
        to: Some("destinatario@exemplo.gov.br".to_string()),
        subject: Some("Fwd: Nota nº 2.306".to_string()),
        date: Some("2026-10-15".to_string()),
    };

    let segments = segment_email(Some(&headers), md, &SegmentationConfig::default());

    // Deve conter Header
    assert_eq!(segments[0].kind, SegmentKind::Header);

    // Deve conter Body com mensagem de abertura
    let body_seg = segments.iter().find(|s| s.kind == SegmentKind::Body);
    assert!(body_seg.is_some(), "Deve conter segmento Body");
    assert!(body_seg.unwrap().content.contains("para conhecimento"));

    // Deve detectar a assinatura com 'Respeitosamente;'
    let sig_seg = segments.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_seg.is_some(), "Deve detectar assinatura com Respeitosamente");
    assert!(sig_seg.unwrap().content.to_lowercase().contains("respeitosamente"));

    // Deve isolar o histórico citado com 'De: ...'
    let quote_seg = segments.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_seg.is_some(), "Deve isolar quoted thread iniciada por De:");
}

#[test]
fn test_fixture_thread_multinivel() {
    let fixture_bytes = include_bytes!("fixtures/fixture_thread_multinivel.eml");
    let raw = RawEmail::new(fixture_bytes.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "servidor@exemplo.gov.br",
        "destinatario@exemplo.gov.br",
        &mut sink,
    )
    .expect("Deve processar thread multinível");

    let md = sanitized.parts.markdown_auto.as_ref().unwrap();
    let segments = segment_email(None, md, &SegmentationConfig::default());

    // Deve conter corpo da mensagem atual
    let body_seg = segments.iter().find(|s| s.kind == SegmentKind::Body);
    assert!(body_seg.is_some(), "Deve conter corpo da mensagem mais recente");
    assert!(body_seg.unwrap().content.contains("Segue em anexo documento solicitado"));

    // Deve detectar assinatura da mensagem mais recente
    let sig_seg = segments.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_seg.is_some(), "Deve detectar assinatura");
    assert!(sig_seg.unwrap().content.contains("Atenciosamente"));

    // Deve isolar histórico anterior
    let quote_seg = segments.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_seg.is_some(), "Deve isolar thread anterior");
}

#[test]
fn test_fixture_boletim_longo_head_tail_chunking() {
    let fixture_bytes = include_bytes!("fixtures/fixture_boletim_longo.eml");
    let raw = RawEmail::new(fixture_bytes.to_vec());
    let mut sink = InMemorySink::new();

    let (sanitized, _result) = Pipeline::process(
        &raw,
        "servidor@exemplo.gov.br",
        "destinatario@exemplo.gov.br",
        &mut sink,
    )
    .expect("Deve processar boletim informativo extenso");

    let md = sanitized.parts.markdown_auto.as_ref().unwrap();

    let config = SegmentationConfig {
        max_body_chunks: 5,
        target_chunk_tokens: 30, // força corte para testar Head-Tail
    };

    let segments = segment_email(None, md, &config);
    let body_segments: Vec<_> = segments.iter().filter(|s| s.kind == SegmentKind::Body).collect();

    // Deve respeitar o teto máximo de 5 blocos para o corpo
    assert_eq!(body_segments.len(), 5);

    // Deve detectar assinatura formal no final do boletim
    let sig_seg = segments.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_seg.is_some(), "Deve detectar assinatura do boletim");
}
