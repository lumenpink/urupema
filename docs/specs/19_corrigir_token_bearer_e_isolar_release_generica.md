# Especificação Técnica — Correção de Credencial Bearer do Cargo e Isolamento de Release Genérica (#19)

- **ID do Ticket:** #19
- **Status:** Concluído (Fase 9: Finish & Close)
- **Autora:** Lumen
- **Alinhamento:** GOST 2.0.0
- **Data:** 2026-11-09

## 1. Contexto & Motivação
Na esteira de CI/CD do `urupema` (execução `#1873`), o job `release` falhou pelos seguintes motivos:
1. **Credencial do Cargo:** No passo `Configure Gambi Cargo Registry`, a gravação do token em `~/.cargo/credentials.toml` omitia o prefixo `Bearer `, exigido pelo registro Forgejo, gerando falhas `401 Unauthorized` na publicação automatizada de crates via `cargo publish`.
2. **Colisão de Releases e Erro 404 em Overwrite:** Em versões estáveis sem hífen (como `0.1.2`), `canonical_tag` e `named_generic_tag` avaliavam para o mesmo valor (`0.1.2`). O passo `Publish Canonical Release` criava a release `0.1.2` e subia o binário. Imediatamente após, o passo `Publish Named Generic Release` tentava publicar na mesma tag `0.1.2`. A action externa (`softprops/action-gh-release@v2`) tentava remover o asset já existente chamando o endpoint do GitHub (`DELETE /repos/{owner}/{repo}/releases/assets/{id}`), que resulta em `404 page not found` na API Forgejo, quebrando a esteira.

## 2. Requisitos Técnicos

### 2.1 Credencial Bearer no Cargo
- No arquivo `.forgejo/workflows/ci.yml`, no passo `Configure Gambi Cargo Registry`:
  ```bash
  if [ -n "$GAMBI_TOKEN" ]; then
    printf '[registries.gambi]\ntoken = "Bearer %s"\n' "$GAMBI_TOKEN" > "$HOME/.cargo/credentials.toml"
  fi
  ```

### 2.2 Isolamento de Release Genérica Nomeada
- O passo `Publish Named Generic Release` deve conter a condição:
  ```yaml
  if: steps.versioning.outputs.named_generic_tag != steps.versioning.outputs.canonical_tag
  ```
- Além disso, a rotina `Delete Generic Releases and Tags` não deve incluir tags que coincidam com a tag canônica (pois a tag canônica nunca deve ser apagada):
  ```python
  canonical = "${{ steps.versioning.outputs.canonical_tag }}"
  generic_tags = [
      t for t in [
          "${{ steps.versioning.outputs.named_generic_tag }}",
          "${{ steps.versioning.outputs.primary_generic_tag }}",
      ] if t != canonical
  ]
  ```

### 2.3 Cobertura de Testes
- Atualizar `tests/test_ci_publish_crate.sh` para validar que o token configurado possui o prefixo `Bearer `.
- Atualizar `tests/test_ci_release_workflow.sh` para validar a guarda condicional evitando a colisão entre `named_generic_tag` e `canonical_tag`.
- Incluir testes de mutantes garantindo que omissões dessas salvaguardas quebrem as asserções.

## 3. Critérios de Aceitação
- Testes `test_ci_publish_crate.sh`, `test_ci_release_workflow.sh` e `test_ci_prune_release_asset.sh` aprovados com testes de mutação.
- 100% de conformidade com `cccp check --strict`.
- Job de release executando de ponta a ponta sem erros de API ou autenticação.
