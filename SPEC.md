# URUPEMA — Especificação Técnica de Domínio & Pipeline MIME

**Versão da Norma:** GOST `1.0.0`  
**Status:** `CONCLUDED` (Branch: `feat/1-domain-pipeline`)  
**Ticket de Referência:** `#1`

---

## 1. Visão Geral do Sistema

O **`urupema`** é um utilitário post-queue e filtro MIME para MTAs Unix (Postfix). Ele é invocado como comando síncrono via transporte `pipe`, lendo uma mensagem bruta em formato RFC 5322 da entrada padrão (`stdin`), processando e higienizando seu conteúdo MIME, e despachando a mensagem reempacotada para injeção limpa no MTA.

```text
               +-------------------------------------------+
               |                 POSTFIX                   |
               +-------------------------------------------+
                                     |
                          stdin (RFC 5322 bruto)
                                     v
               +-------------------------------------------+
               |                 URUPEMA                   |
               |                                           |
               |  1. Parser MIME (mail-parser)             |
               |  2. Extração text/plain & text/html       |
               |  3. Higienização HTML (ammonia)           |
               |  4. Conversão HTML -> Markdown            |
               |  5. Reempacotamento multipart             |
               |  6. Injeção de Header de Rastreio         |
               +-------------------------------------------+
                                     |
                          stdout / subprocess pipe
                                     v
               +-------------------------------------------+
               |  /usr/sbin/sendmail -G -i -f $from $to   |
               +-------------------------------------------+
```

---

## 2. O Pipeline de Processamento RFC 5322

### 2.1 Passo 1: Ingestão e Dissecação Zero-Copy
- Recebe stream de bytes brutos da entrada padrão (`stdin`).
- Utiliza a crate `mail-parser` para dissecar a árvore MIME em tempo linear sem cópias redundantes de memória.
- Preserva todos os cabeçalhos originais (incluindo `Message-ID`, `Date`, `Subject`, `DKIM-Signature`, `Received`), removendo ou atualizando cabeçalhos MIME estruturais caso o corpo seja reescrito (`Content-Type`, `Content-Transfer-Encoding`).

### 2.2 Passo 2: Extração de Partes
A árvore de conteúdo pode se manifestar em 3 topologias canônicas:
1. **Single-part `text/plain`:** Mantida intacta; nenhuma transformação destrutiva.
2. **Single-part `text/html`:**
   - HTML original higienizado via `ammonia`.
   - Conversão para `text/plain` enriquecido (Markdown `txt auto`) via `html2text`.
   - Mensagem convertida em `multipart/alternative`.
3. **`multipart/alternative` ou `multipart/mixed`:**
   - Preserva `text/plain` original se presente.
   - Captura a folha `text/html`.
   - Gera versão `txt auto` e HTML sanitizado.
   - Anexos binários (`application/*`, `image/*`) são preservados integralmente sem modificação de bytes.

### 2.3 Passo 3: Higienização Cirúrgica de HTML
A política de higienização de tags via `ammonia` é estrita:
- **Tags permitidas:** `p`, `br`, `b`, `i`, `strong`, `em`, `a`, `ul`, `ol`, `li`, `blockquote`, `code`, `pre`, `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `table`, `thead`, `tbody`, `tr`, `th`, `td`, `hr`.
- **Atributos permitidos em links (`<a>`):** apenas `href` e `title`. Esquemas restritos a `http`, `https` e `mailto`.
- **Atributos de segurança injetados em links:** `rel="noopener noreferrer"`.
- **Proibições absolutas:**
  - `<script>`, `<iframe>`, `<object>`, `<embed>`, `<applet>`.
  - Estilos externos (`<link rel="stylesheet">`, `@import`).
  - Tags de rastreamento (`<img>` com dimensões 1x1 ou links de tracker).
  - Atributos inline perigosos (`onload`, `onclick`, `style` contendo expressões).

### 2.4 Passo 4: Geração de Markdown (`txt auto`)
- O corpo HTML higienizado é processado por `html2text`.
- Links são preservados na sintaxe padrão Markdown `[texto](url)`.
- Tabelas simples são formatadas em ASCII legível.

### 2.5 Passo 5: Reempacotamento e Rastreabilidade
- Cabeçalho injetado: `X-Urupema-Processed: 1; engine=urupema-0.1.0`.
- O payload gerado é reemitido no formato RFC 5322 em UTF-8.

---

## 3. Arquitetura de Domínio & Interfaces

Seguindo o padrão arquitetural GOST (§4 da norma Rust):

```text
src/
├── domain/
│   ├── mod.rs
│   ├── entity.rs        # RawEmail, SanitizedMessage, EmailParts
│   ├── status.rs        # PipelineStatus { Processed, Bypassed, Rejected }
│   └── ports.rs         # Trait EmailSink
├── application/
│   ├── mod.rs
│   └── use_cases.rs     # process_pipeline(raw, sink)
├── adapters/
│   ├── mod.rs
│   ├── in_memory.rs     # InMemorySink (testes herméticos)
│   ├── sendmail.rs      # SendmailProcessSink (adapter de I/O real)
│   └── cli.rs           # Args clap
└── main.rs              # Composition Root
```

### 3.1 Entidades de Domínio (`domain/entity.rs`)
```rust
pub struct RawEmail {
    pub bytes: Vec<u8>,
}

pub struct EmailParts {
    pub plain_text: Option<String>,
    pub html_text: Option<String>,
    pub markdown_auto: Option<String>,
    pub sanitized_html: Option<String>,
}

pub struct SanitizedMessage {
    pub raw_headers: Vec<(String, String)>,
    pub parts: EmailParts,
    pub final_payload: Vec<u8>,
}
```

### 3.2 Portas (`domain/ports.rs`)
```rust
pub trait EmailSink {
    fn deliver(&mut self, sender: &str, recipient: &str, payload: &[u8]) -> Result<(), EmailSinkError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmailSinkError {
    #[error("Subprocess execution failed: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Process exited with non-zero code: {0}")]
    ProcessFailed(i32),
}
```

### 3.3 Casos de Uso (`application/use_cases.rs`)
```rust
pub fn process_email<S: EmailSink>(
    raw: &[u8],
    sender: &str,
    recipient: &str,
    sink: &mut S,
) -> Result<PipelineSummary, PipelineError>;
```

---

## 4. Estratégia de Testes (TDD Hermético)

Os testes automatizados **NÃO DEVEM** depender de binários externos (`sendmail`) ou de conexões com o Postfix real:
1. `InMemorySink` acumula as mensagens entregues em `Vec<Vec<u8>>` para inspeção imediata.
2. Fixtures RFC 5322 embutidas cobrindo:
   - E-mail texto puro simples (`text/plain`).
   - E-mail puramente HTML com tentativa de script/tracker (`<script>` e pixel 1x1).
   - E-mail multipart com anexo.
   - Caracteres acentuados e codificação Quoted-Printable / Base64.
3. Cobertura de código mantida $\ge 90\%$.

---

## 5. Critérios de Aceite (Ticket #1)

- [x] Documentação prévia da arquitetura consolidada neste `SPEC.md`.
- [x] Configuração do `Cargo.toml` com as crates mínimas (`mail-parser`, `ammonia`, `html2text`, `thiserror`).
- [x] Testes falhos criados para cada caso canônico de e-mail.
- [x] Implementação de `domain/` e `application/` passando 100% nos testes.
- [x] Execução limpa de `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt -- --check`.

---

## 6. Especificação dos Adaptadores de Borda (Ticket #2)

### 6.1 Parser de Linha de Comando (`adapters/cli.rs`)
O binário utilitário é invocado pelo comando `pipe` do Postfix conforme declarado em `master.cf`:
```bash
urupema --sender=${sender} --recipient=${recipient}
```
Argumentos obrigatórios e opcionais via `clap` (derive):
- `--sender` / `-s`: Endereço de envelope do remetente (`${sender}`).
- `--recipient` / `-r`: Endereço de envelope do destinatário (`${recipient}`).
- `--sendmail-bin`: Caminho configurável para o binário de injeção (padrão: `/usr/sbin/sendmail`).

### 6.2 Adaptador de Injeção Sendmail (`adapters/sendmail.rs`)
Implementa o trait `EmailSink` gerenciando o processo filho:
- **Comando:** `<sendmail-bin> -G -i -f <sender> <recipient>`
  - `-G`: Informa ao Postfix que a mensagem é de submissão de gateway/relay.
  - `-i`: Impede encerramento de leitura por ponto solto (`.`).
  - `-f <sender>`: Define o envelope sender original.
- **Streams:**
  - O payload gerado pelo pipeline é escrito diretamente no `stdin` do processo filho.
  - Mensagens informativas ou de diagnóstico do Urupema são emitidas exclusivamente em `stderr`.
- **Códigos de Saída do Processo:**
  - `0`: Sucesso absoluto (mensagem entregue ao MTA).
  - `64` (EX_USAGE): Erro na passagem de argumentos CLI.
  - `75` (EX_TEMPFAIL): Falha transitória ao invocar o sendmail ou ler stdin (Postfix mantém a mensagem na fila).

### 6.3 Critérios de Aceite (Ticket #2)
- [x] Especificação documentada neste `SPEC.md`.
- [x] Teste unitário para parser `clap` validando argumentos válidos e inválidos.
- [x] Implementação de `SendmailSink` respeitando caminhos customizados de binário para testes.
- [x] Conexão no `src/main.rs` lendo de `std::io::stdin()` e despachando para o `SendmailSink`.
- [x] Clippy zero warnings e `cargo fmt --check`.
