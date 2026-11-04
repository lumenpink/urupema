# Especificação Técnica — Publicação de Versão Canônica e Tags Genéricas com Deleção Prévia (#16)

- **ID do Ticket:** #16
- **Status:** Em Progresso (Fase 3: Spec)
- **Autora:** Lumen
- **Alinhamento:** GOST 2.0.0

## 1. Contexto & Motivação
Anteriormente, o job de release do `urupema` publicava apenas na tag estática e rolante `latest`, truncando versões pré-release para a base semântica (`cut -d '-' -f1`).
Isso impedia a identificação clara de versões canônicas imutáveis (tanto para versões estáveis quanto de desenvolvimento) e causava falhas ao sobrescrever assets existentes sem a higienização prévia das tags e releases rolantes.

## 2. Requisitos Técnicos

### 2.1 Resolução das Tags e Canais
A partir do arquivo `VERSION` na raiz do repositório:
- `VERSION_STR`: conteúdo de `VERSION` (ex.: `0.2.0` ou `0.2.0-dev.22+11223344.ababab`).
- `BASE_VERSION`: base semântica antes de qualquer hífen/plus (ex.: `0.2.0`).
- Se `VERSION_STR` contiver `-dev` (ou o evento não for uma tag de release estável `v*`):
  - **Canal:** `nightly` (dev)
  - **Tag Canônica (imutável):** `${VERSION_STR}` (ex.: `0.2.0-dev.22+11223344.ababab`)
  - **Tag Genérica Nomeada:** `${BASE_VERSION}-dev` (ex.: `0.2.0-dev`)
  - **Tag Genérica Principal:** `latest-dev`
- Se `VERSION_STR` for estável (sem `-dev`):
  - **Canal:** `stable`
  - **Tag Canônica (imutável):** `${VERSION_STR}` (ex.: `0.2.0`)
  - **Tag Genérica Nomeada:** `${BASE_VERSION}` (ex.: `0.2.0`)
  - **Tag Genérica Principal:** `latest`

### 2.2 Deleção Prévia Obrigatória das Tags e Releases Genéricas
Antes da publicação dos novos binários:
- As tags genéricas (`latest` / `latest-dev` e `${BASE_VERSION}` / `${BASE_VERSION}-dev`) devem ter sua release e sua tag git prévias removidas via API do Forgejo (`DELETE /repos/{repo}/releases/{id}` e `DELETE /repos/{repo}/tags/{tag}`).
- A tag canônica **NUNCA** é deletada, garantindo rastreabilidade histórica.

### 2.3 Publicação dos Binários
- O binário compilado `target/release/urupema` deve ser publicado sob:
  1. A tag canônica.
  2. A tag genérica nomeada.
  3. A tag genérica principal.

### 2.4 Publicação da Crate no Registro Gambi
- O manifesto `Cargo.toml` deve sincronizar `version` exatamente com `VERSION_STR`, permitindo publicação consistente no registro `gambi`.

## 3. Critérios de Aceitação
- Testes automatizados em `tests/test_ci_release_workflow.sh` validando a resolução de canais, presença da rotina de deleção prévia para tags genéricas e as três tags de publicação.
- Zero warnings em `cargo clippy` e conformidade em `cargo fmt -- --check`.
- 100% de conformidade com `cccp check --strict`.

## 4. Checklist GOST DD-TDD
- [x] **Fase 1: Ticket** — Ticket #16 aberto e sincronizado.
- [x] **Fase 2: Branch** — Branch `feat/16-publicar-versao-canonica-e-genericas-com-delete-previo` criada.
- [x] **Fase 3: Spec** — Especificação técnica redigida.
- [ ] **Fase 4: Red Test** — Testes de auditoria atualizados comprovando falhas no workflow atual.
- [ ] **Fase 5: Green Implement** — Implementada a deleção prévia e a tríade de tags no CI.
- [ ] **Fase 6: Refactor** — Limpeza e padronização dos scripts de automação.
- [ ] **Fase 7: Mutation Test** — Validação de mutantes nos testes do workflow.
- [ ] **Fase 8: Quality Gate** — Verificação rigorosa com clippy, fmt, tests e cccp check.
- [ ] **Fase 9: Finish & Close** — Atualização do changelog e fechamento do ciclo.
