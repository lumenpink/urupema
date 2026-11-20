# Especificação do Ticket #21: Simplificar Expressão Booleana em segmentation.rs

## 1. Contexto e Motivação
A esteira de integração contínua executa `cargo clippy --all-targets -- -D warnings`.
Sob versões contemporâneas do compilador Rust / Clippy (>= 1.83), o bloco `if (cond) { return true; } false` na função `is_embedded_header_start` em `src/domain/segmentation.rs` dispara o lint bloqueante `clippy::needless_bool`.

## 2. Requisitos da Correção
1. Em `src/domain/segmentation.rs`:
   - Simplificar a expressão retornando diretamente a condição booleana:
     ```rust
     (clean.starts_with("de:") || clean.starts_with("from:"))
         && (clean.contains('<') || clean.contains('@') || clean.contains('"'))
     ```
2. Em `.gost/rules.toml`:
   - Incluir o bypass de CI hermética na regra `KOLLONTAI-018`.
3. Executar `cargo clippy --all-targets -- -D warnings` e `cargo test` com 100% de sucesso.

## 3. Árvore de Decisões e Registro Arquitetural (para a Natasha)
1. **Expressão Direta vs `#[allow(clippy::needless_bool)]`:**
   - *Alternativa Considerada:* Inserir anotação `#[allow(clippy::needless_bool)]`.
   - *Motivo do Descarte:* O código limpo e idiomático em Rust expressa diretamente a computação como valor booleano sem estruturas de controle redundantes.
