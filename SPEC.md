# URUPEMA — Especificação Técnica de Domínio & Pipeline MIME

**Versão da Norma:** GOST `1.0.0`  
**Status:** `DRAFT` (Branch: `fix/12-markdown-sanitization-payload-selection`)  
**Ticket de Referência:** `#12` (Anteriores: `#6`, `#4`, `#5`, `#1`)

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

### 2.4 Passo 4: Geração de Markdown (`txt auto`) e Preservação de Quebras
- O corpo HTML higienizado é processado por `html2text`.
- **Largura Flexível / Responsiva:** Em vez de quebras artificiais rígidas a 80 colunas (que corrompem o fluxo de leitura em visualizadores modernos e telas dinâmicas), a conversão utiliza uma largura ampla (unwrapped, e.g. 10.000 colunas), permitindo que o cliente de exibição controle o wrap de texto livremente.
- **Preservação de Quebras Intencionais:** Quebras de parágrafo (`<p>`) são preservadas como parágrafos separados (`\n\n`), e quebras de linha duras (`<br>`, `<br/>`) são mantidas como quebras de linha (`\n`), garantindo integridade visual em endereços, listas, versos e blocos de assinatura.
- Links são preservados na sintaxe padrão Markdown `[texto](url)`.
- Tabelas simples são formatadas em ASCII legível.

#### 2.4.1 Sanitização de Ruídos de Markdown (Ticket #12)
- Newsletters e comunicados frequentes (ex.: GitHub Universe, Canal Meio) geram distorções com divisores ASCII corrompidos, especialmente sequências repetitivas de underlines (`___________`).
- O sanitizador de ruídos aplica normalização determinística:
  - Sequências de 3 ou mais underlines repetitivos (`_{3,}`) são colapsadas ou removidas quando isoladas em linhas de separação ruidosas.
  - Marcadores corrompidos ou fragmentos órfãos de formatação são saneados.
  - O texto resultante mantém a legibilidade e previne quebras na renderização de Markdown.

#### 2.4.2 Heurística de Seleção de Payload (Ticket #12)
- E-mails primordialmente visuais (ex.: Pinterest) possuem HTML que, após o parser, produz conteúdo vazio (apenas espaços em branco) ou excessivamente poluído com fragmentos vazios e links truncados, enquanto o `original_plain` multipart contém o texto semântico real da mensagem.
- A heurística avalia:
  1. Se `markdown_auto` resultante for vazio ou tiver apenas espaços em branco, e houver `original_plain`, recorrer imediatamente ao `original_plain`.
  2. Se `markdown_auto` tiver baixíssima densidade textual útil comparada a `original_plain` (indicando HTML puramente cosmético ou quebrado), recorrer ao `original_plain`.
  3. Caso contrário, quando o HTML possui conteúdo textual substantivo, utilizar o `markdown_auto` devidamente sanitizado.

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
│   ├── segmentation.rs  # EmailSegment, SegmentKind, fatiamento semantico
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

---

## 7. Segmentação Semântica e Fatiamento Estrutural (Ticket #4)

Para alimentar o motor de busca semântica vetorial do Lauda (operando com embeddings eficientes na CPU sem competir com Baquara na GPU), o Urupema disseca a mensagem limpa em blocos semânticos tipados.

### 7.1 Tipos e Estruturas de Domínio (`domain/segmentation.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// Metadados resumidos (De, Para, Assunto, Data)
    Header,
    /// Chunks do corpo principal da mensagem
    Body,
    /// Fecho de despedida e assinatura do remetente
    Signature,
    /// Histórico citado de respostas anteriores
    QuotedThread,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailSegment {
    pub kind: SegmentKind,
    pub content: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct SegmentationConfig {
    /// Número máximo de chunks para o corpo (default: 5)
    pub max_body_chunks: usize,
    /// Limite aproximado de tokens/palavras por chunk (default: 350)
    pub target_chunk_tokens: usize,
}
```

### 7.2 Regras de Detecção Trilíngue (PT, EN, ES)

1. **Fechos e Assinaturas (`SegmentKind::Signature`)**:
   - **RFC 3676**: Marcador universal `-- ` (dois traços seguidos de espaço opcional e quebra de linha).
   - **Português (pt)**: `atenciosamente`, `att.`, `att,`, `atte.`, `cordialmente`, `cordiais saudações`, `abraços`, `um abraço`, `forte abraço`, `saudações`, `cumprimentos`, `grato`, `obrigado`, `obrigada`, `desde já agradeço`.
   - **Inglês (en)**: `best regards`, `warm regards`, `regards`, `sincerely`, `yours sincerely`, `yours faithfully`, `cheers`, `best`, `all the best`, `thanks`, `thank you`, `many thanks`, `respectfully`.
   - **Espanhol (es)**: `atentamente`, `atte.`, `atte,`, `cordialmente`, `saludos cordiales`, `saludos`, `un saludo`, `un abrazo`, `gracias`, `muchas gracias`, `quedo a su disposición`, `quedo atento`, `quedo atenta`.

2. **Citação e Histórico (`SegmentKind::QuotedThread`)**:
   - **Marcador Universal**: Linhas iniciadas por `>` ou `>>`.
   - **Português (pt)**: `em ..., ... escreveu:`, `no dia ..., ... escreveu:`, `----- mensagem original -----`, `----- mensagem encaminhada -----`.
   - **Inglês (en)**: `on ..., ... wrote:`, `----- original message -----`, `----- forwarded message -----`.
   - **Espanhol (es)**: `el ..., ... escribió:`, `en ..., ... escribió:`, `----- mensaje original -----`, `----- mensaje reenviado -----`.

### 7.3 Estratégia de Fatiamento Cabeça-Cauda (Head-Tail Chunking)

Para evitar que e-mails anômalos ou hiper-longos inflacionem a matriz vetorial sem agregar valor semântico:
- **Teto do Body:** Até `max_body_chunks` (padrão 5).
- **E-mails curtos e médios ($\le 5$ chunks):** Fatiamento sequencial completo sem omissão de partes.
- **E-mails longos (> 5 chunks):**
  - **Head (Cabeça):** Preserva os primeiros 4 blocos sequenciais a partir do início.
  - **Tail (Cauda):** O 5º bloco é extraído a partir do fim do corpo contando de trás para frente (~350 tokens finais).
  - Garante que a abertura (objetivo) e o desfecho (conclusões e rodapés) estejam sempre presentes no índice vetorial.

### 7.4 Critérios de Aceite (Ticket #4)
- [x] Tipagem `EmailSegment`, `SegmentKind`, `HeaderMeta` e `SegmentationConfig` implementadas em `src/domain/segmentation.rs`.
- [x] Detecção trilíngue (Português, Inglês e Espanhol) para fechos/assinaturas e marcadores de resposta citada.
- [x] Suporte ao delimitador universal de assinatura RFC 3676 (`-- `) e prefixo de citação (`>`).
- [x] Algoritmo de fatiamento Cabeça-Cauda (Head-Tail Chunking) garantindo teto de 5 blocos sem perda da cauda.
- [x] Suíte de testes em `tests/segmentation_test.rs` cobrindo todas as variantes de idioma, casos combinados e corpos vazios.
- [x] Quality Gate 100% verde (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`).

---

## 8. Fixtures Reais Anonimizadas e Suíte de Regressão (Ticket #6)

Para garantir que a dissecação semântica e o pipeline de higienização operem com máxima fidelidade em cenários do mundo real (Zimbra, Outlook, webmail corporativo/governamental), a suíte de testes incorpora fixtures reais anonimizadas (deface).

### 8.1 Política de Anonimização & Deface
As fixtures contêm e-mails reais que passam por processo estrito de higienização antes de commit:
- **Nomes Próprios:** Substituídos por personas fictícias neutras (`Servidor Exemplo`, `Fulano de Tal`, `Maria Silva`).
- **Endereços de E-mail:** Mapeados deterministicamente para o domínio RFC 2606 `exemplo.gov.br` ou `exemplo.org`.
- **Identificadores Sensíveis:** Telefones, ramais, CPFs e matrículas substituídos por dígitos fictícios mascarados.
- **Estruturas Preservadas:** A árvore MIME completa, headers RFC 5322, tags HTML, estilos embutidos e espaçamentos originais são preservados com precisão cirúrgica para reproduzir as exatas idiossincrasias dos clientes emissores.

### 8.2 Arquétipos Canônicos em `tests/fixtures/`
1. `fixture_militar_fwd.eml`: E-mail formal com encaminhamento, fecho `Respeitosamente;`, cabeçalho `De: ...\nPara: ...` no corpo e árvore multipart.
2. `fixture_thread_multinivel.eml`: Conversa encadeada entre 3 órgãos distintos, contendo histórico citado com aviso de LGPD e assinatura padrão RFC 3676 (`-- `).
3. `fixture_boletim_longo.eml`: Informativo institucional volumoso (> 50 KB) com múltiplas seções, testando o teto do algoritmo Head-Tail Chunking.

### 8.3 Aprimoramentos Heurísticos no Domínio (`domain/segmentation.rs`)
- **Fechos em Português:** Inclusão de `respeitosamente` (comum no meio administrativo, militar e jurídico).
- **Cabeçalhos de Resposta/Encaminhamento:** Reconhecimento de blocos embutidos estilo Webmail/Zimbra/Outlook iniciados por `De: ` / `From: ` seguidos de `Para: ` / `To: ` ou `Enviadas: ` / `Sent: `.

### 8.4 Critérios de Aceite (Ticket #6)
- [x] Utilitário de anonimização e deface implementado e documentado (`scripts/anonymize_eml.py`).
- [x] Arquivos de fixture `.eml` armazenados em `tests/fixtures/`.
- [x] Heurísticas de fecho (`respeitosamente`) e cabeçalhos Webmail (`De: ...\nPara: ...`) implementadas no `segmentation.rs`.
- [x] Testes de regressão em `tests/fixture_test.rs` validando segmentação e pipeline contra todas as fixtures.
- [x] Quality Gate 100% verde (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`).



