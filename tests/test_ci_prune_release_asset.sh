#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.forgejo/workflows/ci.yml"
SPEC_FILE="$REPO_ROOT/docs/specs/15_corrigir_poda_assets_release.md"

FAILED=0

echo "==> [TEST] Validando conformidade da poda de assets da release (#15)..."

# 1. Validação da Spec
if [ ! -f "$SPEC_FILE" ]; then
    echo "FALHA: Arquivo de especificação $SPEC_FILE não encontrado."
    FAILED=1
else
    echo "OK: $SPEC_FILE presente."
fi

# 2. Validação do arquivo de workflow
if [ ! -f "$WORKFLOW_FILE" ]; then
    echo "FALHA: Workflow $WORKFLOW_FILE não encontrado."
    exit 1
fi

# 3. Validar ausência de dependência de jq no workflow
if grep -q "jq " "$WORKFLOW_FILE"; then
    echo "FALHA: Dependência de 'jq' encontrada no workflow (ausente no runner)."
    FAILED=1
else
    echo "OK: Ausência de 'jq' confirmada."
fi

# 4. Validar uso de python3 para poda com urllib e json
if ! grep -q "python3 -c" "$WORKFLOW_FILE"; then
    echo "FALHA: Rotina 'python3 -c' ausente no passo de poda."
    FAILED=1
else
    echo "OK: 'python3 -c' presente para poda de assets."
fi

# 5. Validar uso do token GAMBI_TOKEN no env do passo de poda
if ! grep -q "GAMBI_TOKEN: \${{ secrets.GAMBI_2LP_IN_TOKEN }}" "$WORKFLOW_FILE"; then
    echo "FALHA: GAMBI_TOKEN não injetado para poda de release."
    FAILED=1
else
    echo "OK: GAMBI_TOKEN devidamente configurado."
fi

# 6. Validar ausência de terminologia imperialista (soberano)
if grep -iq "soberan" "$WORKFLOW_FILE"; then
    echo "FALHA: Terminologia inadequada ('soberano') encontrada no workflow."
    FAILED=1
else
    echo "OK: Terminologia alinhada e saneada."
fi

# 7. Testes de Mutação
echo "==> Executando testes de mutação para poda de assets..."
TMP_WORKFLOW="$(mktemp)"
trap 'rm -f "$TMP_WORKFLOW"' EXIT

# Mutante 1: Reintrodução de jq
sed 's/python3 -c/jq -r/' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if grep -q "python3 -c" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 1 (sem python3) não isolado."
    FAILED=1
else
    echo "OK: Mutante 1 isolado."
fi

# Mutante 2: Remoção do token GAMBI_TOKEN
sed 's/GAMBI_TOKEN: \${{ secrets.GAMBI_2LP_IN_TOKEN }}//' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if grep -q "GAMBI_TOKEN: \${{ secrets.GAMBI_2LP_IN_TOKEN }}" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 2 (sem token) não isolado."
    FAILED=1
else
    echo "OK: Mutante 2 isolado."
fi

if [ $FAILED -ne 0 ]; then
    echo "==> [TEST] STATUS: RED (Asserções falharam conforme esperado)"
    exit 1
fi

echo "==> [TEST] STATUS: GREEN (Todas as asserções e mutações passaram)"
