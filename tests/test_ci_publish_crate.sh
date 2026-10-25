#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.forgejo/workflows/ci.yml"
SPEC_FILE="$REPO_ROOT/docs/specs/10_publish_crate_gambi.md"

FAILED=0

echo "==> [TEST] Validando conformidade da publicação da crate urupema no registro gambi (#10)..."

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

# 3. Validar declaração do registro gambi
if ! grep -q '\[registries\.gambi\]' "$WORKFLOW_FILE"; then
    echo "FALHA: Configuração [registries.gambi] ausente no workflow."
    FAILED=1
else
    echo "OK: Configuração [registries.gambi] presente."
fi

# 4. Validar sparse index da gambi
if ! grep -q 'sparse+https://gambi.2lp.in/api/packages/lumen/cargo/' "$WORKFLOW_FILE"; then
    echo "FALHA: Índice sparse da gambi ausente no workflow."
    FAILED=1
else
    echo "OK: Índice sparse configurado."
fi

# 5. Validar uso do secret GAMBI_2LP_IN_TOKEN
if ! grep -q 'GAMBI_2LP_IN_TOKEN' "$WORKFLOW_FILE"; then
    echo "FALHA: Secret GAMBI_2LP_IN_TOKEN ausente no workflow."
    FAILED=1
else
    echo "OK: Secret GAMBI_2LP_IN_TOKEN referenciado."
fi

# 6. Validar comando cargo publish com registry gambi
if ! grep -q 'cargo publish --registry gambi' "$WORKFLOW_FILE"; then
    echo "FALHA: Comando 'cargo publish --registry gambi' ausente no workflow."
    FAILED=1
else
    echo "OK: Comando cargo publish --registry gambi configurado."
fi

if [ "$FAILED" -ne 0 ]; then
    echo "==> [FAIL] Teste de conformidade falhou."
    exit 1
else
    echo "==> [PASS] Todas as validações da publicação no registro gambi foram aprovadas."
fi
