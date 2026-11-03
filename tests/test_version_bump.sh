#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION_FILE="$REPO_ROOT/VERSION"
CARGO_TOML="$REPO_ROOT/Cargo.toml"
SPEC_FILE="$REPO_ROOT/docs/specs/14_bump_version_and_publish_gambi.md"

FAILED=0

echo "==> [TEST] Validando elevação de versão para 0.1.1 (#14)..."

# 1. Spec presente
if [ ! -f "$SPEC_FILE" ]; then
    echo "FALHA: Especificação $SPEC_FILE não encontrada."
    FAILED=1
else
    echo "OK: $SPEC_FILE presente."
fi

# 2. VERSION base = 0.1.1
if [ ! -f "$VERSION_FILE" ]; then
    echo "FALHA: Arquivo $VERSION_FILE não encontrado."
    FAILED=1
else
    CANONICAL_VERSION=$(head -n 1 "$VERSION_FILE" | cut -d '-' -f1 | tr -d ' \r\n')
    if [ "$CANONICAL_VERSION" != "0.1.1" ]; then
        echo "FALHA: Versão em $VERSION_FILE é '$CANONICAL_VERSION', esperado '0.1.1'."
        FAILED=1
    else
        echo "OK: Versão canônica em VERSION é 0.1.1."
    fi
fi

# 3. Cargo.toml version = 0.1.1
if [ ! -f "$CARGO_TOML" ]; then
    echo "FALHA: Arquivo $CARGO_TOML não encontrado."
    FAILED=1
else
    CARGO_VER=$(grep -m1 '^version = ' "$CARGO_TOML" | cut -d '"' -f2)
    if [ "$CARGO_VER" != "0.1.1" ]; then
        echo "FALHA: Versão em $CARGO_TOML é '$CARGO_VER', esperado '0.1.1'."
        FAILED=1
    else
        echo "OK: Versão em Cargo.toml é 0.1.1."
    fi
fi

# 4. Testes de Mutação
echo "==> Executando testes de mutação para verificação de versão..."
TMP_MUTANT=$(mktemp)
trap 'rm -f "$TMP_MUTANT"' EXIT

# Mutante 1: Versão 0.1.0 deve falhar
printf "0.1.0\n" > "$TMP_MUTANT"
MUT_VER=$(head -n 1 "$TMP_MUTANT" | cut -d '-' -f1 | tr -d ' \r\n')
if [ "$MUT_VER" = "0.1.1" ]; then
    echo "FALHA: Mutante 1 não detectou divergência."
    exit 1
fi
echo "OK: Mutante 1 (versão antiga 0.1.0) detectado e rejeitado."

# Mutante 2: Versão arbitrária 0.9.9 deve falhar
printf "0.9.9\n" > "$TMP_MUTANT"
MUT_VER2=$(head -n 1 "$TMP_MUTANT" | cut -d '-' -f1 | tr -d ' \r\n')
if [ "$MUT_VER2" = "0.1.1" ]; then
    echo "FALHA: Mutante 2 não detectou divergência."
    exit 1
fi
echo "OK: Mutante 2 (versão arbitrária 0.9.9) detectado e rejeitado."

if [ "$FAILED" -ne 0 ]; then
    echo "==> [TEST] STATUS: RED (Falha esperada na Fase 4)"
    exit 1
fi

echo "==> [TEST] STATUS: GREEN (Todas as asserções passaram)"
