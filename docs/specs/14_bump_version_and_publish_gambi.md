# Spec: Publicação da Crate Urupema 0.1.1 no Registro Gambi (#14)

## Contexto
O ticket #12 introduziu a higienização de ruídos e divisores de underlines em Markdown (`sanitize_markdown_noise`) e a heurística de seleção de payload (`is_markdown_empty_or_polluted`).
Contudo, como a versão permaneceu em `0.1.0` (já previamente publicada no registro Gambi em 25/10/2026), a esteira não publicou os novos binários e crates, impedindo que consumidores como `lauda-server` utilizassem a correção.

## Objetivo
1. Elevar a versão canônica no arquivo `VERSION` e no manifesto `Cargo.toml` para `0.1.1`.
2. Validar conformidade de publicação no registro Gambi (`sparse+https://gambi.2lp.in/api/packages/lumen/cargo/`).
3. Assegurar conformidade com a norma GOST v2.0.0 (incluindo teto de 50 linhas no `README.md`).

## Critérios de Aceitação
- `VERSION` contém exatamente `0.1.1` (ou SemVer 2.0.0 com base `0.1.1`).
- `Cargo.toml` declara `version = "0.1.1"`.
- Script de teste automatizado `tests/test_version_bump.sh` valida a versão canônica e mutações com assertividade.
- Quality Gate 100% verde (`cargo fmt`, `cargo clippy`, `cargo test`, `cccp check --strict`).
- Publicação bem-sucedida no registro Gambi via `cargo publish --registry gambi`.
