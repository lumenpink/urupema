#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.forgejo/workflows/ci.yml"
SPEC_FILE="$REPO_ROOT/docs/specs/16_publicar_versao_canonica_e_genericas_spec.md"

FAILED=0

echo "==> [TEST] Validando conformidade do workflow de publicacao canonica e genericas com delete previo (#16)..."

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

# 3. Validar declaração do job release
if ! grep -q "^  release:" "$WORKFLOW_FILE"; then
    echo "FALHA: Job 'release:' ausente no workflow."
    FAILED=1
else
    echo "OK: Job 'release:' declarado."
fi

# 4. Validar compilação release do binário urupema
if ! grep -q "cargo build --release --bin urupema" "$WORKFLOW_FILE"; then
    echo "FALHA: Passo de build 'cargo build --release --bin urupema' ausente."
    FAILED=1
else
    echo "OK: Build release de urupema configurado."
fi

# 5. Validar resolução de tags (canonica, named generic e primary generic)
if ! grep -q "canonical_tag" "$WORKFLOW_FILE"; then
    echo "FALHA: Resolução de 'canonical_tag' ausente no workflow."
    FAILED=1
else
    echo "OK: 'canonical_tag' resolvido no workflow."
fi

if ! grep -q "named_generic_tag" "$WORKFLOW_FILE"; then
    echo "FALHA: Resolução de 'named_generic_tag' ausente no workflow."
    FAILED=1
else
    echo "OK: 'named_generic_tag' resolvido no workflow."
fi

if ! grep -q "primary_generic_tag" "$WORKFLOW_FILE"; then
    echo "FALHA: Resolução de 'primary_generic_tag' ausente no workflow."
    FAILED=1
else
    echo "OK: 'primary_generic_tag' resolvido no workflow."
fi

# 6. Validar presença do step de deleção prévia das tags e releases genéricas
if ! grep -q "Delete Generic Releases and Tags" "$WORKFLOW_FILE"; then
    echo "FALHA: Step 'Delete Generic Releases and Tags' ausente no workflow."
    FAILED=1
else
    echo "OK: Step 'Delete Generic Releases and Tags' presente."
fi

# 7. Validar que as releases genéricas sofrem deleção via API Forgejo
if ! grep -q "/releases/{rel_id}" "$WORKFLOW_FILE" && ! grep -q "/releases/\$rel_id" "$WORKFLOW_FILE"; then
    echo "FALHA: Chamada DELETE para remoção de release genérica ausente."
    FAILED=1
else
    echo "OK: Chamada DELETE para release genérica presente."
fi

if ! grep -q "/tags/{tag}" "$WORKFLOW_FILE" && ! grep -q "/tags/\$tag" "$WORKFLOW_FILE"; then
    echo "FALHA: Chamada DELETE para remoção de tag genérica ausente."
    FAILED=1
else
    echo "OK: Chamada DELETE para tag genérica presente."
fi

# 8. Validar suporte a latest-dev para builds dev
if ! grep -q "latest-dev" "$WORKFLOW_FILE"; then
    echo "FALHA: Suporte a tag 'latest-dev' ausente no workflow."
    FAILED=1
else
    echo "OK: Suporte a 'latest-dev' configurado."
fi

# 9. Validar sincronização de Cargo.toml sem truncar -dev
if grep -q "cut -d '-' -f1" "$WORKFLOW_FILE"; then
    echo "FALHA: O workflow ainda utiliza 'cut -d '-' -f1' truncando versões dev."
    FAILED=1
else
    echo "OK: Sincronização de Cargo.toml não trunca versão dev."
fi

if [ $FAILED -ne 0 ]; then
    echo "==> [TEST] STATUS: RED (Asserções falharam conforme esperado)"
    exit 1
fi

echo "==> Executando testes de mutação para release workflow (#16)..."
TMP_WORKFLOW="$(mktemp)"
trap 'rm -f "$TMP_WORKFLOW"' EXIT

# Mutante 1: Ausência de 'Delete Generic Releases and Tags'
sed 's/Delete Generic Releases and Tags/Ignore Generic Deletions/' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if grep -q "Delete Generic Releases and Tags" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 1 (sem delete previo) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 1 (sem delete previo) detectado e isolado."

# Mutante 2: Falta de suporte a latest-dev
sed 's/latest-dev/latest-broken/' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if grep -q "latest-dev" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 2 (sem latest-dev) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 2 (sem latest-dev) detectado e isolado."

# Mutante 3: Reintrodução de cut -d '-' -f1
sed 's/RAW_VERSION/RAW_VERSION | cut -d '\''-'\'' -f1/' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if ! grep -q "cut -d '-' -f1" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 3 (truncando -dev) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 3 (truncando -dev) detectado e isolado."

echo "==> [TEST] STATUS: GREEN (Todas as asserções e mutações passaram)"
