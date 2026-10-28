//! Higienização de ruídos e seleção de payload de mensagens.

/// Limpa ruídos visuais, marcadores corrompidos e sequências repetitivas de underlines
/// comumente injetados por newsletters (ex.: comunicados do GitHub Universe e Canal Meio).
pub fn sanitize_markdown_noise(input: &str) -> String {
    let mut cleaned_lines = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        // 1. Linhas compostas exclusivamente por 3 ou mais underlines e espaços (divisores ruidosos de newsletters)
        if trimmed.len() >= 3
            && trimmed.chars().all(|c| c == '_' || c == ' ')
            && trimmed.contains("___")
        {
            continue;
        }

        // 2. Substituir sequências de 4 ou mais underlines dentro da linha
        let mut sanitized_line = String::with_capacity(line.len());
        let mut under_count = 0;

        for c in line.chars() {
            if c == '_' {
                under_count += 1;
            } else {
                if under_count >= 4 {
                    // Descarta sequências de underlines repetitivos
                } else {
                    for _ in 0..under_count {
                        sanitized_line.push('_');
                    }
                }
                under_count = 0;
                sanitized_line.push(c);
            }
        }

        if under_count < 4 {
            for _ in 0..under_count {
                sanitized_line.push('_');
            }
        }

        cleaned_lines.push(sanitized_line);
    }

    // Normaliza múltiplas linhas em branco consecutivas (no máximo uma linha vazia entre parágrafos)
    let mut result = Vec::new();
    let mut consecutive_empty = 0;

    for line in cleaned_lines {
        if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty <= 1 {
                result.push(line);
            }
        } else {
            consecutive_empty = 0;
            result.push(line);
        }
    }

    result.join("\n").trim().to_string()
}

/// Heurística para detecção de Markdown vazio ou excessivamente poluído/degradado
/// em relação ao texto puro original (caso típico de e-mails imagéticos como Pinterest).
pub fn is_markdown_empty_or_polluted(markdown: &str, original_plain: Option<&str>) -> bool {
    let md_trimmed = markdown.trim();
    if md_trimmed.is_empty() {
        return true;
    }

    if let Some(plain) = original_plain {
        let plain_trimmed = plain.trim();
        if !plain_trimmed.is_empty() {
            let md_alpha_count = md_trimmed.chars().filter(|c| c.is_alphanumeric()).count();
            let plain_alpha_count = plain_trimmed
                .chars()
                .filter(|c| c.is_alphanumeric())
                .count();

            // Se o texto puro original tem conteúdo substantivo e o Markdown gerado do HTML
            // tiver densidade alfanumérica insignificante ou desproporcionalmente menor (< 20%)
            if plain_alpha_count >= 20
                && (md_alpha_count < 15 || md_alpha_count * 5 < plain_alpha_count)
            {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_preserves_markdown_emphasis() {
        let input = "Este é um texto com _itálico_ e __negrito__ legítimo.";
        assert_eq!(sanitize_markdown_noise(input), input);
    }

    #[test]
    fn test_sanitize_removes_repetitive_underlines_lines() {
        let input = "Título\n\n____________________\n\nTexto após divisor.";
        let cleaned = sanitize_markdown_noise(input);
        assert!(!cleaned.contains("___"));
        assert!(cleaned.contains("Título"));
        assert!(cleaned.contains("Texto após divisor."));
    }

    #[test]
    fn test_sanitize_removes_inline_four_or_more_underlines() {
        let input = "Ruído____remover e _____mais.";
        let cleaned = sanitize_markdown_noise(input);
        assert!(!cleaned.contains("____"));
        assert_eq!(cleaned, "Ruídoremover e mais.");
    }

    #[test]
    fn test_sanitize_collapses_excessive_blank_lines() {
        let input = "Linha 1\n\n\n\n\n\nLinha 2";
        let cleaned = sanitize_markdown_noise(input);
        assert_eq!(cleaned, "Linha 1\n\nLinha 2");
    }

    #[test]
    fn test_is_markdown_empty_or_polluted_scenarios() {
        // Vazio ou apenas whitespace
        assert!(is_markdown_empty_or_polluted("", None));
        assert!(is_markdown_empty_or_polluted("   \n\t  ", None));

        // Markdown rico sem texto puro original
        assert!(!is_markdown_empty_or_polluted(
            "Conteúdo perfeitamente válido com mais de vinte caracteres aqui.",
            None
        ));

        // Markdown degradado vs texto puro rico (Pinterest)
        let rich_plain = "Confira a nova seleção de produtos artesanais disponíveis para você.";
        assert!(is_markdown_empty_or_polluted(
            "Link: ",
            Some(rich_plain)
        ));

        // Ambos ricos
        assert!(!is_markdown_empty_or_polluted(
            "Confira a nova seleção de produtos artesanais no nosso catálogo online.",
            Some(rich_plain)
        ));

        // Plain curto / sem relevância não deve disparar fallback de markdown válido
        assert!(!is_markdown_empty_or_polluted(
            "Mensagem curta.",
            Some("Curto")
        ));
    }
}
