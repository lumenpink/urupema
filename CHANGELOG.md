# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Sanitização de ruídos visuais e divisores repetitivos de underlines em Markdown e heurística de seleção de payload com fallback para text/plain (closes #12).
- Configuração de `[profile.release]` com LTO e strip, e linking estático `+crt-static` para musl libc (closes #8).

### Added
- Publicação da crate da biblioteca urupema no registro soberano gambi (closes #10).
- Publicação automática de release linux-x86_64 sob a tag latest na Gambi (closes #9).
- Inicialização do repositório em conformidade com o padrão GOST.
