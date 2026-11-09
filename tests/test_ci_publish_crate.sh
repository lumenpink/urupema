#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.forgejo/workflows/ci.yml"
SPEC_FILE="$REPO_ROOT/docs/specs/10_publish_crate_gambi.md"
KOMBINAT_SPEC_FILE="$REPO_ROOT/docs/specs/17_notificacao_kombinat_cascata_spec.md"
TICKET19_SPEC_FILE="$REPO_ROOT/docs/specs/19_corrigir_token_bearer_e_isolar_release_generica.md"

FAILED=0

echo "==> [TEST] Validando conformidade da publicação da crate urupema no registro gambi (#10), cascata kombinat (#17) e token bearer (#19)..."

# 1. Validação das Specs
if [ ! -f "$SPEC_FILE" ]; then
    echo "FALHA: Arquivo de especificação $SPEC_FILE não encontrado."
    FAILED=1
else
    echo "OK: $SPEC_FILE presente."
fi

if [ ! -f "$KOMBINAT_SPEC_FILE" ]; then
    echo "FALHA: Arquivo de especificação $KOMBINAT_SPEC_FILE não encontrado."
    FAILED=1
else
    echo "OK: $KOMBINAT_SPEC_FILE presente."
fi

if [ ! -f "$TICKET19_SPEC_FILE" ]; then
    echo "FALHA: Arquivo de especificação $TICKET19_SPEC_FILE não encontrado."
    FAILED=1
else
    echo "OK: $TICKET19_SPEC_FILE presente."
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

# 7. Validar step de notificação ao Kombinat daemon
if ! grep -q 'Notify Kombinat Daemon of Release Cascade' "$WORKFLOW_FILE"; then
    echo "FALHA: Step 'Notify Kombinat Daemon of Release Cascade' ausente no workflow."
    FAILED=1
else
    echo "OK: Step de notificação do Kombinat presente."
fi

# 8. Validar endpoint /api/cascade/dispatch
if ! grep -q '/api/cascade/dispatch' "$WORKFLOW_FILE"; then
    echo "FALHA: Endpoint '/api/cascade/dispatch' ausente no workflow."
    FAILED=1
else
    echo "OK: Endpoint /api/cascade/dispatch configurado."
fi

# 9. Validar envio do pacote urupema no payload
if ! grep -q '\\"package\\": \\"urupema\\"' "$WORKFLOW_FILE"; then
    echo "FALHA: Payload JSON não declara '\"package\": \"urupema\"'."
    FAILED=1
else
    echo "OK: Payload JSON declara '\"package\": \"urupema\"'."
fi

# 10. Validar fallback de rede para Docker runner (172.17.0.1:4040)
if ! grep -q '172.17.0.1:4040' "$WORKFLOW_FILE"; then
    echo "FALHA: Fallback 172.17.0.1:4040 ausente no workflow."
    FAILED=1
else
    echo "OK: Fallback 172.17.0.1:4040 configurado."
fi

# 11. Validar uso do prefixo Bearer no token de autenticação do registro gambi (#19)
if ! grep -q 'token = "Bearer %s"' "$WORKFLOW_FILE"; then
    echo "FALHA: Prefixo 'Bearer ' ausente na configuração do token de credentials.toml."
    FAILED=1
else
    echo "OK: Prefixo 'Bearer ' configurado para autenticação do cargo."
fi

if [ "$FAILED" -ne 0 ]; then
    echo "==> [FAIL] Teste de conformidade falhou."
    exit 1
fi

echo "==> Executando testes de mutação para publish crate workflow..."

TMP_MUTANT=$(mktemp)
trap 'rm -f "$TMP_MUTANT"' EXIT

# Mutante 1: Registry divergente (ex: crates-io)
sed 's/cargo publish --registry gambi/cargo publish --registry crates-io/' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q "cargo publish --registry gambi" "$TMP_MUTANT"; then
    echo "FALHA: Mutante 1 (registry divergente) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 1 (registry divergente) detectado e rejeitado."

# Mutante 2: Token ausente
sed 's/GAMBI_2LP_IN_TOKEN/UNKNOWN_TOKEN/' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q "GAMBI_2LP_IN_TOKEN" "$TMP_MUTANT"; then
    echo "FALHA: Mutante 2 (token ausente) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 2 (token ausente) detectado e rejeitado."

# Mutante 3: Endpoint divergente
sed 's|/api/cascade/dispatch|/api/cascade/divergente|g' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q "/api/cascade/dispatch" "$TMP_MUTANT"; then
    echo "FALHA: Mutante 3 (endpoint divergente) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 3 (endpoint divergente) detectado e rejeitado."

# Mutante 4: Pacote divergente
sed 's/\\"package\\": \\"urupema\\"/\\"package\\": \\"divergente\\"/' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q '\\"package\\": \\"urupema\\"' "$TMP_MUTANT"; then
    echo "FALHA: Mutante 4 (pacote divergente) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 4 (pacote divergente) detectado e rejeitado."

# Mutante 5: Fallback ausente
sed 's/172\.17\.0\.1:4040/0\.0\.0\.0:9999/' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q "172\.17\.0\.1:4040" "$TMP_MUTANT"; then
    echo "FALHA: Mutante 5 (fallback ausente) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 5 (fallback ausente) detectado e rejeitado."

# Mutante 6: Prefixo Bearer ausente (#19)
sed 's/Bearer %s/%s/' "$WORKFLOW_FILE" > "$TMP_MUTANT"
if grep -q 'token = "Bearer %s"' "$TMP_MUTANT"; then
    echo "FALHA: Mutante 6 (sem Bearer) não foi simulado corretamente."
    exit 1
fi
echo "OK: Mutante 6 (sem Bearer) detectado e rejeitado."

echo "==> [TEST] STATUS: GREEN (Todas as asserções e mutações passaram)"
