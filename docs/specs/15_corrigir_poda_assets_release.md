# Especificação Técnica — Correção da Poda de Assets na Release Forgejo (#15)

- **ID do Ticket:** #15
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 2.0.0

## 1. Contexto & Motivação
Na esteira de CI/CD do repositório Urupema (`.forgejo/workflows/ci.yml`), o job `release` compila o binário `linux-x86_64` e o publica na tag `latest` da forja (`gambi.2lp.in`).
Entretanto, a execução falhou porque:
1. O passo de poda prévia do binário antigo dependia do utilitário `jq`, ausente na imagem base do runner (`gost-ci-cross`), falhando silenciosamente e mantendo o binário anterior na release `latest`.
2. O passo utilizava `secrets.GITHUB_TOKEN` sem declará-lo explicitamente no bloco `env:`, além de a credencial efetiva no ambiente ser `secrets.GAMBI_2LP_IN_TOKEN`.
3. Ao tentar publicar com o asset existente, a action `softprops/action-gh-release@v2` tentou excluir o asset chamando a rota da API do GitHub (`DELETE /repos/{owner}/{repo}/releases/assets/{asset_id}`), a qual não existe no Forgejo (que exige `/repos/{owner}/{repo}/releases/{id}/assets/{attachment_id}`), retornando HTTP 404 e abortando a esteira.

## 2. Requisitos Técnicos
1. **Eliminação de Dependência Externa (`jq`):**
   - O passo de limpeza prévia deve utilizar exclusivamente Python 3 (`python3 -c "..."`), já disponível no ambiente base (`Python 3.11+`), manipulando JSON e requisições HTTP via biblioteca padrão (`urllib.request`, `json`, `os`, `sys`).
2. **Autenticação Adequada:**
   - Injetar no bloco `env:` a variável `GAMBI_TOKEN: ${{ secrets.GAMBI_2LP_IN_TOKEN }}` (com fallback opcional para `GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}`).
3. **Poda Conforme Rota Forgejo:**
   - Localizar a release `latest` via `GET /api/v1/repos/{owner}/{repo}/releases/tags/latest`.
   - Caso exista o asset `urupema`, invocar `DELETE /api/v1/repos/{owner}/{repo}/releases/{id}/assets/{asset_id}` com o cabeçalho `Authorization: token <TOKEN>`.
4. **Saneamento Terminológico:**
   - Remover qualquer termo de cunho imperialista dos textos de título, etapas e corpos do workflow e da release.
5. **Cobertura de Testes Automatizados:**
   - Criar ou atualizar teste de automação validando a ausência de `jq` no passo de poda e a presença da rotina em Python 3 com as variáveis e cabeçalhos apropriados.
   - Incluir testes de mutação para garantir a detecção de regressões.

## 3. Critérios de Aceitação
- Testes automatizados passando com cobertura de mutação (status GREEN).
- Validação estrita via `cccp check --strict`.
- Job de release sem referências quebradas ou rotas incompatíveis.
