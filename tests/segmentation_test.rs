use urupema::domain::{segment_email, HeaderMeta, SegmentKind, SegmentationConfig};

#[test]
fn test_segmentation_header_and_short_body() {
    let headers = HeaderMeta {
        from: Some("remetente@exemplo.org".to_string()),
        to: Some("destinatario@exemplo.org".to_string()),
        subject: Some("Reunião Semanal".to_string()),
        date: Some("2026-10-15 10:00:00".to_string()),
    };

    let body =
        "Olá a todos,\n\nConfirmamos nossa reunião para amanhã às 14h no auditório principal.";

    let segments = segment_email(Some(&headers), body, &SegmentationConfig::default());

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind, SegmentKind::Header);
    assert!(segments[0].content.contains("De: remetente@exemplo.org"));
    assert!(segments[0].content.contains("Assunto: Reunião Semanal"));

    assert_eq!(segments[1].kind, SegmentKind::Body);
    assert!(segments[1].content.contains("Confirmamos nossa reunião"));
}

#[test]
fn test_trilingual_signature_detection() {
    let config = SegmentationConfig::default();

    // 1. Português
    let email_pt =
        "Olá camarada,\n\nSegue a ata da reunião.\n\nAbraços,\nLumen Pink\nArquiteto de Software";
    let segs_pt = segment_email(None, email_pt, &config);
    let sig_pt = segs_pt.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_pt.is_some(), "Deve detectar assinatura em PT");
    let content = &sig_pt.unwrap().content;
    assert!(content.contains("Abraços"));
    assert!(content.contains("Lumen Pink"));

    // 2. Inglês
    let email_en =
        "Hello team,\n\nThe deployment was successful.\n\nBest regards,\nJohn Doe\nSRE Lead";
    let segs_en = segment_email(None, email_en, &config);
    let sig_en = segs_en.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_en.is_some(), "Deve detectar assinatura em EN");
    let content = &sig_en.unwrap().content;
    assert!(content.contains("Best regards"));
    assert!(content.contains("John Doe"));

    // 3. Espanhol
    let email_es =
        "Estimados colegas,\n\nAdjunto el informe mensual.\n\nSaludos cordiales,\nMaría González";
    let segs_es = segment_email(None, email_es, &config);
    let sig_es = segs_es.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_es.is_some(), "Deve detectar assinatura em ES");
    let content = &sig_es.unwrap().content;
    assert!(content.contains("Saludos cordiales"));
    assert!(content.contains("María González"));

    // 4. RFC 3676
    let email_rfc = "Bom dia.\n\nSegue o código.\n\n-- \nLumen\nhttps://lumen.pink";
    let segs_rfc = segment_email(None, email_rfc, &config);
    let sig_rfc = segs_rfc.iter().find(|s| s.kind == SegmentKind::Signature);
    assert!(sig_rfc.is_some(), "Deve detectar assinatura RFC 3676");
    let content = &sig_rfc.unwrap().content;
    assert!(content.contains("https://lumen.pink"));
}

#[test]
fn test_trilingual_quoted_thread_detection() {
    let config = SegmentationConfig::default();

    // 1. Português: "Em ..., ... escreveu:"
    let email_pt = "Concordo plenamente.\n\nEm qua., 14 de out. de 2026 às 10:00, Carlos escreveu:\n> Proponho adiarmos o prazo.";
    let segs_pt = segment_email(None, email_pt, &config);
    let quote_pt = segs_pt.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_pt.is_some(), "Deve isolar quoted thread em PT");
    assert!(quote_pt
        .unwrap()
        .content
        .contains("Proponho adiarmos o prazo"));

    // 2. Inglês: "On ..., ... wrote:"
    let email_en = "Looks good to me.\n\nOn Wed, Oct 14, 2026 at 10:00 AM, Jane wrote:\n> Can we review this tomorrow?";
    let segs_en = segment_email(None, email_en, &config);
    let quote_en = segs_en.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_en.is_some(), "Deve isolar quoted thread em EN");
    assert!(quote_en
        .unwrap()
        .content
        .contains("Can we review this tomorrow?"));

    // 3. Espanhol: "El ..., ... escribió:"
    let email_es = "De acuerdo.\n\nEl mié., 14 de oct. de 2026 a las 10:00, Pedro escribió:\n> ¿Revisamos el informe hoy?";
    let segs_es = segment_email(None, email_es, &config);
    let quote_es = segs_es.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_es.is_some(), "Deve isolar quoted thread em ES");
    assert!(quote_es
        .unwrap()
        .content
        .contains("¿Revisamos el informe hoy?"));

    // 4. Linhas citadas com prefixo `>`
    let email_gt =
        "Resposta rápida.\n\n> Linha 1 de contexto anterior\n> Linha 2 de contexto anterior";
    let segs_gt = segment_email(None, email_gt, &config);
    let quote_gt = segs_gt.iter().find(|s| s.kind == SegmentKind::QuotedThread);
    assert!(quote_gt.is_some(), "Deve isolar citação com >");
    assert!(quote_gt
        .unwrap()
        .content
        .contains("Linha 1 de contexto anterior"));
}

#[test]
fn test_head_tail_chunking_on_huge_email() {
    let config = SegmentationConfig {
        max_body_chunks: 5,
        target_chunk_tokens: 50, // valor pequeno para forçar corte controlado
    };

    // Cria 10 parágrafos distintos
    let mut paragraphs = Vec::new();
    for i in 1..=10 {
        paragraphs.push(format!(
            "Parágrafo {i}: Este é um texto representativo de volume razoável para teste de fatiamento do parágrafo número {i} com bastante conteúdo semântico."
        ));
    }
    let huge_body = paragraphs.join("\n\n");

    let segments = segment_email(None, &huge_body, &config);
    let body_segments: Vec<_> = segments
        .iter()
        .filter(|s| s.kind == SegmentKind::Body)
        .collect();

    // Deve respeitar o limite máximo de 5 chunks para o corpo
    assert_eq!(body_segments.len(), 5);

    // O primeiro chunk (Head 1) deve conter o primeiro parágrafo
    assert!(body_segments[0].content.contains("Parágrafo 1"));

    // O último chunk (Tail) deve conter o último parágrafo (contado de trás pra frente)
    assert!(body_segments[4].content.contains("Parágrafo 10"));
}

#[test]
fn test_all_segments_combined_in_single_email() {
    let headers = HeaderMeta {
        from: Some("maria@empresa.com".to_string()),
        to: Some("joao@empresa.com".to_string()),
        subject: Some("Atualização do Projeto".to_string()),
        date: Some("2026-10-15 11:30:00".to_string()),
    };

    let full_email = "Olá João,\r\n\r\n\
                      Os testes de homologação foram concluídos com sucesso.\r\n\r\n\
                      Atenciosamente,\r\n\
                      Maria Silva\r\n\
                      Diretora de Engenharia\r\n\r\n\
                      Em qua., 14 de out. de 2026 às 15:00, João escreveu:\r\n\
                      > Como estão os testes de homologação?";

    let segments = segment_email(Some(&headers), full_email, &SegmentationConfig::default());

    assert_eq!(segments.len(), 4);
    assert_eq!(segments[0].kind, SegmentKind::Header);
    assert_eq!(segments[0].index, 0);

    assert_eq!(segments[1].kind, SegmentKind::Body);
    assert_eq!(segments[1].index, 1);
    assert!(segments[1].content.contains("Os testes de homologação"));

    assert_eq!(segments[2].kind, SegmentKind::Signature);
    assert_eq!(segments[2].index, 2);
    assert!(segments[2].content.contains("Atenciosamente"));
    assert!(segments[2].content.contains("Maria Silva"));

    assert_eq!(segments[3].kind, SegmentKind::QuotedThread);
    assert_eq!(segments[3].index, 3);
    assert!(segments[3].content.contains("Como estão os testes"));
}

#[test]
fn test_empty_or_whitespace_body() {
    let config = SegmentationConfig::default();

    let segs_empty = segment_email(None, "", &config);
    assert!(segs_empty.is_empty());

    let segs_spaces = segment_email(None, "   \n\n   \t  \n", &config);
    assert!(segs_spaces.is_empty());

    let headers = HeaderMeta {
        from: Some("test@domain.com".to_string()),
        ..Default::default()
    };
    let segs_header_only = segment_email(Some(&headers), "   ", &config);
    assert_eq!(segs_header_only.len(), 1);
    assert_eq!(segs_header_only[0].kind, SegmentKind::Header);
}
