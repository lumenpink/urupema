# urupema (Filtro Post-Queue MIME & Higienizador para Postfix)

> Utilitário post-queue em Rust síncrono para dissecação cirúrgica de MIME, conversão em Markdown e sanitização estrita de e-mails recebidos pelo Postfix.

[![GOST Compliance](https://img.shields.io/badge/GOST-1.0.0-crimson)](https://codeberg.org/lumenpink/gost)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
[![Website](https://img.shields.io/badge/portal-urupema.2lp.in-red)](https://urupema.2lp.in)

---

## ⚡ Visão Geral

O **`urupema`** roda como um binário utilitário no servidor de correio, invocado pelo transporte `pipe` do Postfix após a mensagem entrar na fila. Ele lê o payload RFC 5322 bruto da entrada padrão (`stdin`), desseca a árvore MIME e higieniza o conteúdo:

1. **Zero-Copy MIME:** Inspeciona cabeçalhos, anexos e corpos via `mail-parser`.
2. **Preservação de Texto Puro:** Mantém o `text/plain` original intacto.
3. **Markdown Limpo (`txt auto`):** Converte blocos HTML em texto formatado preservando links via `html2text`.
4. **Higienização Cirúrgica:** Remove scripts, rastreadores e estilos externos via `ammonia` com injeção de `rel="noopener noreferrer"`.
5. **Prevenção de Loops:** Injeta e reconhece o cabeçalho `X-Urupema-Processed: 1`.
6. **Despacho Direto:** Reinjeta no MTA chamando `/usr/sbin/sendmail -G -i -f "$sender" "$recipient"`.

---

## 🛠️ Configuração no Postfix (`/etc/postfix/master.cf`)

Adicione o serviço de transporte `pipe`:

```text
urupema  unix  -  n  n  -  -  pipe
  flags=Rq user=urupema argv=/usr/local/bin/urupema
  --sender=${sender} --recipient=${recipient}
```

E configure o roteamento no `main.cf`:

```text
# Exemplo para filtrar e-mails locais ou de domínio específico:
transport_maps = hash:/etc/postfix/transport
```

---

## 🧪 Testes e Qualidade

O projeto adota rigor **DD-TDD** conforme a norma [GOST](https://codeberg.org/lumenpink/gost):

```bash
# Executar todos os testes de unidade e herméticos
cargo test --all-targets --all-features

# Validar linter com tolerância zero a avisos
cargo clippy --all-targets --all-features -- -D warnings

# Checar formatação determinística
cargo fmt --all -- --check
```

---

## 🎨 Vitrine e Documentação

A documentação estática adota o tema visual **Trash Polka** (halftones, máquina de escrever, carmesim e contrastes brutais):
* **Portal online:** [urupema.2lp.in](https://urupema.2lp.in)
* **Arquivos:** [`site/index.html`](./site/index.html) e [`site/_headers`](./site/_headers)
* **Especificação de Domínio:** [`SPEC.md`](./SPEC.md)

---

## 📄 Licença

Este projeto é software livre e de código aberto distribuído sob a licença **Apache-2.0**:

| Componente | Licença | Arquivo |
| :--- | :--- | :--- |
| **Código-Fonte e Core** | Apache-2.0 | [`LICENSE`](LICENSE) |
| **Logotipos e Identidade** | CC-BY-ND-4.0 | [`LICENSE-LOGO`](LICENSE-LOGO) |
| **Mídias e Ilustrações** | CC-BY-SA-4.0 | [`LICENSE-MEDIA`](LICENSE-MEDIA) |
