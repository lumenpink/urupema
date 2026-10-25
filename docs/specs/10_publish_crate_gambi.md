# Especificação Técnica — Publicação Automática da Crate urupema no Registro Gambi (#10)

- **ID do Ticket:** #10
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0

## 1. Contexto & Motivação
O `urupema` exporta uma biblioteca Rust com domínios fundamentais de sanitização, parsing MIME e segmentação textual de e-mails (`adapters`, `application`, `domain`).
O aplicativo irmão `lauda` (através do `lauda-index` e `lauda-server`) depende dessas definições para indexação BM25 e busca semântica.
Para evitar a necessidade de clonar o repositório fonte a cada ciclo de compilação ou teste de integração, o `urupema` deve publicar automaticamente a sua biblioteca no registro de pacotes Cargo soberano (`gambi.2lp.in`), sob o identificador de registro `gambi`.

## 2. Requisitos Técnicos
1. **Configuração do Registro Soberano:**
   - Adicionar configuração em `~/.cargo/config.toml`:
     ```toml
     [registries.gambi]
     index = "sparse+https://gambi.2lp.in/api/packages/lumen/cargo/"
     ```
   - Injetar o token de autenticação em `~/.cargo/credentials.toml` a partir do secret `GAMBI_2LP_IN_TOKEN`:
     ```toml
     [registries.gambi]
     token = "${{ secrets.GAMBI_2LP_IN_TOKEN }}"
     ```
2. **Passo de Publicação:**
   - No job de release / deploy da esteira `.forgejo/workflows/ci.yml`, adicionar o step de publicação da crate:
     ```bash
     cargo publish --registry gambi --allow-dirty || echo "::warning::Versao ja publicada ou falha tolerada no registry."
     ```
   - O step deve ser executado condicionado à presença da credencial `GAMBI_2LP_IN_TOKEN`.
3. **Sincronização de Versão:**
   - Garantir que a versão publicada reflita a versão canônica do manifesto `Cargo.toml` / `VERSION`.

## 3. Critérios de Aceitação
- Teste de automação em `tests/test_ci_publish_crate.sh` validando a existência e conformidade do step de publicação e credencial `GAMBI_2LP_IN_TOKEN` no `.forgejo/workflows/ci.yml`.
- Execução limpa do teste de verificação e aprovação do Quality Gate via `cccp check --strict`.
