# urupema (Filtro Post-Queue MIME & Higienizador para Postfix)

> Utilitário post-queue em Rust síncrono para dissecação de MIME, conversão em Markdown e sanitização estrita de e-mails recebidos pelo Postfix.

[![GOST Compliance](https://img.shields.io/badge/GOST-2.0.0-crimson)](https://gostlumen.2lp.in)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
[![Website](https://img.shields.io/badge/portal-urupema.2lp.in-red)](https://urupema.2lp.in)

---

## ⚡ Visão Geral

O **`urupema`** roda como um binário utilitário no servidor de correio via transporte `pipe` do Postfix. Ele lê o payload RFC 5322 da entrada padrão (`stdin`), desseca a árvore MIME e higieniza o conteúdo:

1. **Zero-Copy MIME:** Inspeciona cabeçalhos e corpos via `mail-parser`.
2. **Preservação de Texto Puro:** Mantém o `text/plain` original intacto.
3. **Markdown Limpo (`txt auto`):** Converte HTML em Markdown via `html2text` e expurga divisores de newsletters (`sanitize_markdown_noise`).
4. **Higienização Cirúrgica:** Remove scripts e rastreadores via `ammonia` injetando `rel="noopener noreferrer"`.
5. **Despacho Direto:** Reinjeta no MTA chamando `/usr/sbin/sendmail -G -i -f "$sender" "$recipient"`.

---

## 🛠️ Configuração Rápida no Postfix

No `/etc/postfix/master.cf`:

```text
urupema  unix  -  n  n  -  -  pipe
  flags=Rq user=urupema argv=/usr/local/bin/urupema
  --sender=${sender} --recipient=${recipient}
```

---

## 🧪 Qualidade e Especificação

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Para a especificação completa de domínio e arquitetura, consulte [`SPEC.md`](./SPEC.md). Licenciado sob [Apache-2.0](./LICENSE).
