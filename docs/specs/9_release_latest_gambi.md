# Especificação Técnica — Publicação Automática de Release linux-x86_64 com Tag latest na Gambi (#9)

- **ID do Ticket:** #9
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0

## 1. Contexto & Motivação
O binário `urupema` é utilizado como sanitizador MIME e post-queue filter no servidor de e-mail e como dependência do ecossistema Lauda. Atualmente, os consumidores necessitam clonar e compilar o código fonte repetidamente.
O objetivo desta especificação é definir o job de release automatizado no workflow Forgejo CI (`.forgejo/workflows/ci.yml`) para compilar o binário em modo `--release` para a arquitetura `linux-x86_64` e publicá-lo sob a tag `latest` na forja soberana (`gambi.2lp.in`).

## 2. Requisitos Técnicos
1. **Gatilho de Execução:**
   - Push na branch `main`.
2. **Dependência de Sucesso:**
   - O job `release` depende da aprovação prévia do job `test` (`needs: [test]`).
3. **Compilação do Binário:**
   - `cargo build --release --bin urupema`.
   - Otimizações ativas conforme perfil definido em `Cargo.toml`.
4. **Publicação da Release:**
   - Utilização de `softprops/action-gh-release@v2`.
   - `tag_name: "latest"`.
   - Anexo único do artefato `target/release/urupema`.
5. **Saneamento:**
   - Remoção de jobs órfãos/legados (`trigger-gost-ci`).

## 3. Critérios de Aceitação
- Teste de automação em `tests/test_ci_release_workflow.sh` validando a existência e as diretivas do job de release.
- Validação estrita via `cccp check --strict`.
