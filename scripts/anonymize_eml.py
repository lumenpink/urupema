#!/usr/bin/env python3
"""
anonymize_eml.py - Sanitizador e anonymizer determinístico de e-mails RFC 5322.

Substitui dados sensíveis (PII: e-mails, nomes de pessoas, domínios reais,
telefones, ramais e matrículas) por marcadores fictícios padronizados,
mantendo a exata estrutura MIME (headers, boundaries, HTML e quebras).
"""

import sys
import re
import os

REPLACEMENTS = [
    # E-mails e domínios
    (r'mail\.pm\.sc\.gov\.br', 'mail.exemplo.gov.br'),
    (r'intranet\.pm\.sc\.gov\.br', 'intranet.exemplo.gov.br'),
    (r'pm\.sc\.gov\.br', 'exemplo.gov.br'),
    (r'iprev\.sc\.gov\.br', 'exemplo.gov.br'),
    (r'contato\.ufsc\.br', 'exemplo.edu.br'),
    (r'[a-zA-Z0-9_.+-]+@sib\.com', 'bot@exemplo.com'),
    (r'[a-zA-Z0-9_.+-]+@lumen\.pink', 'usuario@exemplo.org'),
    (r'[a-zA-Z0-9_.+-]+@lohn\.in', 'mta@exemplo.org'),
    (r'[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+', 'contato@exemplo.org'),

    # Nomes específicos encontrados nas amostras
    (r'TC Vieira', 'TC Santos'),
    (r'1º Sgt Alves', '1º Sgt Silva'),
    (r'Den Márcia R\. da Silveira', 'Ana Cláudia de Oliveira'),
    (r'Márcia R\. da Silveira', 'Ana Cláudia de Oliveira'),
    (r'JAILSON AURELIO FRANZEN', 'CARLOS ALBERTO NOGUEIRA'),
    (r'Jailson Aurelio Franzen', 'Carlos Alberto Nogueira'),
    (r'André Felipe Amaral Silva', 'Marcos Aurélio Pereira'),
    (r'Jean Carlos Medeiros', 'Eduardo Ramos Soares'),
    (r'THIAGO JOSE HENTCHEN', 'LUCAS GABRIEL FERREIRA'),
    (r'Thiago José Hentchen', 'Lucas Gabriel Ferreira'),

    # Telefones e ramais
    (r'\(48\)\s*3229-6235', '(48) 3000-0001'),
    (r'\(48\)\s*3229-6019', '(48) 3000-0002'),
    (r'\(48\)\s*3665-9921', '(48) 3000-0003'),
    (r'Ramal\s*\d+', 'Ramal 1234'),

    # CEPs e Endereços específicos
    (r'Rua Visconde de Ouro Preto, 549', 'Rua das Palmeiras, 100'),
    (r'Rua Major Costa, 221', 'Avenida Central, 500'),
    (r'88\.020-040', '88000-000'),
    (r'88020-400', '88000-000'),
]

def anonymize_text(text: str) -> str:
    res = text
    for pattern, rep in REPLACEMENTS:
        res = re.sub(pattern, rep, res, flags=re.IGNORECASE)
    return res

def stub_attachments(msg):
    """Substitui payloads pesados de anexos binários por stubs compactos."""
    for part in msg.walk():
        ct = part.get_content_type()
        if ct not in ('text/plain', 'text/html', 'multipart/alternative', 'multipart/mixed', 'multipart/related'):
            part.set_payload(b'[STUB_ATTACHMENT_BYTES]')
            if 'Content-Transfer-Encoding' in part:
                part.replace_header('Content-Transfer-Encoding', '7bit')

def main():
    if len(sys.argv) < 3:
        print(f"Uso: {sys.argv[0]} <arquivo_origem.eml> <arquivo_destino.eml>")
        sys.exit(1)

    import email
    src = sys.argv[1]
    dst = sys.argv[2]

    with open(src, 'rb') as f:
        msg = email.message_from_binary_file(f)

    stub_attachments(msg)
    raw_bytes = msg.as_bytes()

    try:
        content = raw_bytes.decode('utf-8')
    except UnicodeDecodeError:
        content = raw_bytes.decode('latin-1')

    anonymized = anonymize_text(content)

    with open(dst, 'wb') as f:
        f.write(anonymized.encode('utf-8'))

    print(f"✓ Anonymized: {src} -> {dst} ({len(anonymized)} bytes)")

if __name__ == '__main__':
    main()
