# Especificação Técnica — Notificação ao Kombinat Daemon para Cascata Downstream no Lauda (#17)

- **ID do Ticket:** #17
- **Status:** Concluído (Fase 8: Quality Gate)
- **Autora:** Lumen
- **Alinhamento:** GOST 2.0.0
- **Data:** 2026-11-05

## 1. Contexto & Motivação
A cada nova versão ou build publicado do `urupema` no registro Gambi, os pacotes dependentes downstream precisam ser atualizados de forma contínua e autônoma.
O `kombinat` (daemon de integração contínua e cascata GitOps) expõe o endpoint `POST /api/cascade/dispatch` para receber eventos de release e propagar as dependências declaradas no arquivo `kombinat.toml`.
O pacote downstream principal do `urupema` é o `lauda` (nos sub-crates `lauda-server` e `lauda-index`).
Esta especificação define a integração do step de notificação ao daemon `kombinat` no workflow `.forgejo/workflows/ci.yml`.

## 2. Requisitos Técnicos

### 2.1 Posicionamento no Workflow de CI
- O step `"Notify Kombinat Daemon of Release Cascade"` deve ser executado no job `release` imediatamente após `"Publish Crate to Gambi Registry"`.

### 2.2 Resolução da Versão para Dependência Cargo
- A versão enviada ao `kombinat` não deve conter metadados de build com o delimitador `+` (pois o Cargo não aceita metadados de build como restrição de versão de dependência):
  ```bash
  VERSION=$(head -n 1 VERSION | cut -d'+' -f1 | tr -d ' \r\n')
  ```

### 2.3 Disparo HTTP e Resiliência de Rede
- Disparo via `curl` efetuando requisição `POST` com cabeçalho `Content-Type: application/json` e payload:
  ```json
  {"package": "urupema", "version": "${VERSION}"}
  ```
- **Resiliência e Fallback:**
  1. Tentativa primária: `http://127.0.0.1:4040/api/cascade/dispatch` (execução local / host runner).
  2. Tentativa secundária: `http://172.17.0.1:4040/api/cascade/dispatch` (execução em container Docker CI / bridge gateway).
  3. Tolerância: Em caso de falha em ambos os endpoints, deve ser emitido um warning formatado (`::warning::...`) sem abortar ou falhar o pipeline de CI.

## 3. Critérios de Aceitação
- Testes automatizados em `tests/test_ci_publish_crate.sh` validando a presença e corretude da etapa `"Notify Kombinat Daemon of Release Cascade"`, endpoints e tolerância a falhas.
- Zero warnings em `cargo clippy` e conformidade em `cargo fmt -- --check`.
- 100% de conformidade com `cccp check --strict`.

## 4. Checklist GOST DD-TDD
- [x] **Fase 1: Ticket** — Ticket #17 aberto no Forgejo.
- [x] **Fase 2: Branch** — Branch `feat/17-integrar-notificacao-de-cascata-ao-kombinat-para-lauda` criada.
- [x] **Fase 3: Spec** — Especificação técnica redigida.
- [x] **Fase 4: Red Test** — Auditoria em `tests/test_ci_publish_crate.sh` falhando pela ausência do step.
- [x] **Fase 5: Green Implement** — Step implementado no `.forgejo/workflows/ci.yml`.
- [x] **Fase 6: Refactor** — Limpeza e consistência no workflow.
- [x] **Fase 7: Mutation Test** — Mutantes no script de teste garantindo integridade das regras.
- [x] **Fase 8: Quality Gate** — Validação com fmt, clippy, testes e cccp check.
- [x] **Fase 9: Finish & Close** — Atualização do changelog e fechamento do ciclo.
