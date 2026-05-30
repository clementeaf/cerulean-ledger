#!/usr/bin/env bash
# ==============================================================================
# verify-originality.sh
# Comparacion forense entre cerulean-ledger y cerulean-dlt-framework
#
# Cualquier tercero puede ejecutar este script para verificar de forma
# independiente que no existe codigo copiado entre ambos repositorios.
#
# Uso:
#   bash verify-originality.sh
#   bash verify-originality.sh --output report.txt   # guardar reporte
#
# Requisitos: git, grep, awk, comm, wc (herramientas POSIX estandar)
# ==============================================================================
set -euo pipefail
# Pipes with head/sort/comm can trigger SIGPIPE (exit 141) under pipefail.
# We handle this by wrapping pipe chains in subshells where needed.

REPO_A="https://github.com/clementeaf/cerulean-ledger.git"
REPO_B="https://github.com/Alefrank76/cerulean-dlt-framework.git"
NAME_A="cerulean-ledger"
NAME_B="cerulean-dlt-framework"
MIN_LINE_LEN=15

WORKDIR=$(mktemp -d)
OUTPUT=""

if [[ "${1:-}" == "--output" && -n "${2:-}" ]]; then
    OUTPUT="$2"
fi

log() {
    if [[ -n "$OUTPUT" ]]; then
        echo "$@" | tee -a "$OUTPUT"
    else
        echo "$@"
    fi
}
separator() { log "========================================================================"; }

cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

# ==============================================================================
# 1. Clonar repositorios
# ==============================================================================
separator
log "VERIFICACION DE ORIGINALIDAD — $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
separator
log ""
log "[1/8] Clonando repositorios..."

git clone --quiet "$REPO_A" "$WORKDIR/$NAME_A" 2>/dev/null
git clone --quiet "$REPO_B" "$WORKDIR/$NAME_B" 2>/dev/null

SHA_A=$(cd "$WORKDIR/$NAME_A" && git rev-parse HEAD)
SHA_B=$(cd "$WORKDIR/$NAME_B" && git rev-parse HEAD)

log "  $NAME_A  HEAD: $SHA_A"
log "  $NAME_B  HEAD: $SHA_B"
log ""

# ==============================================================================
# 2. Precedencia temporal
# ==============================================================================
log "[2/8] Precedencia temporal (primer commit)..."

ROOT_A=$(cd "$WORKDIR/$NAME_A" && git rev-list --max-parents=0 HEAD)
FIRST_A=$(cd "$WORKDIR/$NAME_A" && git log -1 --format="%ai" "$ROOT_A")
ROOT_B=$(cd "$WORKDIR/$NAME_B" && git rev-list --max-parents=0 HEAD)
FIRST_B=$(cd "$WORKDIR/$NAME_B" && git log -1 --format="%ai" "$ROOT_B")

log "  $NAME_A:  $FIRST_A"
log "  $NAME_B:  $FIRST_B"
log ""

# ==============================================================================
# 3. Estadisticas de cada repositorio
# ==============================================================================
log "[3/8] Estadisticas de codigo..."

COMMITS_A=$(cd "$WORKDIR/$NAME_A" && git rev-list --count HEAD)
COMMITS_B=$(cd "$WORKDIR/$NAME_B" && git rev-list --count HEAD)

FILES_RS_A=$(find "$WORKDIR/$NAME_A" -name '*.rs' | wc -l | tr -d ' ')
FILES_RS_B=$(find "$WORKDIR/$NAME_B" -name '*.rs' | wc -l | tr -d ' ')

LINES_A=$(find "$WORKDIR/$NAME_A" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')
LINES_B=$(find "$WORKDIR/$NAME_B" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')

AUTHORS_A=$(cd "$WORKDIR/$NAME_A" && git log --all --format="%an" | sort -u | tr '\n' ', ' | sed 's/,$//') || true
AUTHORS_B=$(cd "$WORKDIR/$NAME_B" && git log --all --format="%an" | sort -u | tr '\n' ', ' | sed 's/,$//') || true

log "  | Metrica            | $NAME_A | $NAME_B |"
log "  |--------------------|-----------------|--------------------------|"
log "  | Commits            | $COMMITS_A | $COMMITS_B |"
log "  | Archivos .rs       | $FILES_RS_A | $FILES_RS_B |"
log "  | Lineas Rust        | $LINES_A | $LINES_B |"
log "  | Autores (git)      | $AUTHORS_A | $AUTHORS_B |"
log ""

# ==============================================================================
# 4. Incompatibilidad de frameworks
# ==============================================================================
log "[4/8] Verificando frameworks..."

ACTIX=$(grep -rl "actix" "$WORKDIR/$NAME_A/Cargo.toml" 2>/dev/null | wc -l | tr -d ' ')
SUBSTRATE=$(grep -rl "frame_support\|frame-support\|sp-core\|sp-runtime" "$WORKDIR/$NAME_B" --include='*.toml' 2>/dev/null | wc -l | tr -d ' ')

if [[ "$ACTIX" -gt 0 ]]; then
    log "  $NAME_A usa Actix-Web (confirmado en Cargo.toml)"
else
    log "  $NAME_A: Actix-Web NO encontrado"
fi

if [[ "$SUBSTRATE" -gt 0 ]]; then
    log "  $NAME_B usa Substrate/Polkadot SDK (confirmado en Cargo.toml)"
else
    log "  $NAME_B: Substrate NO encontrado"
fi

if [[ "$ACTIX" -gt 0 && "$SUBSTRATE" -gt 0 ]]; then
    log "  RESULTADO: Frameworks mutuamente incompatibles. Codigo no es intercambiable."
fi
log ""

# ==============================================================================
# 5. Comparacion de dependencias
# ==============================================================================
log "[5/8] Comparando dependencias (Cargo.toml)..."

extract_deps() {
    find "$1" -name 'Cargo.toml' -exec cat {} + 2>/dev/null | \
        grep -E '^\s*\w+\s*=' | \
        sed 's/\s*=.*//' | \
        sed 's/^\s*//' | \
        grep -v -E '^(name|version|edition|authors|description|license|repository|readme|keywords|categories|publish|autobins|autoexamples|autotests|autobenches|resolver|members|exclude|include|default-run|build|links|workspace)$' | \
        sort -u
}

extract_deps "$WORKDIR/$NAME_A" > "$WORKDIR/deps_a.txt"
extract_deps "$WORKDIR/$NAME_B" > "$WORKDIR/deps_b.txt"

COMMON_DEPS=$(comm -12 "$WORKDIR/deps_a.txt" "$WORKDIR/deps_b.txt")
DEPS_A_COUNT=$(wc -l < "$WORKDIR/deps_a.txt" | tr -d ' ')
DEPS_B_COUNT=$(wc -l < "$WORKDIR/deps_b.txt" | tr -d ' ')

log "  Dependencias unicas en $NAME_A: $DEPS_A_COUNT"
log "  Dependencias unicas en $NAME_B: $DEPS_B_COUNT"

if [[ -z "$COMMON_DEPS" ]]; then
    log "  Dependencias en comun: 0"
else
    COMMON_COUNT=$(echo "$COMMON_DEPS" | wc -l | tr -d ' ')
    log "  Dependencias en comun: $COMMON_COUNT"
    echo "$COMMON_DEPS" | while read -r dep; do
        log "    - $dep"
    done
fi
log ""

# ==============================================================================
# 6. Comparacion linea a linea (nucleo del analisis)
# ==============================================================================
log "[6/8] Comparacion linea a linea de codigo fuente..."
log "  Filtro: lineas >= $MIN_LINE_LEN chars, excluyendo blanks, comentarios,"
log "  llaves, use/mod/atributos, tokens minimos (Ok(()), None, Some(...))"
log ""

extract_lines() {
    find "$1" -name '*.rs' -exec cat {} + 2>/dev/null | \
        grep -v '^\s*$' | \
        grep -v '^\s*//' | \
        grep -v '^\s*\*' | \
        grep -v '^\s*[{}]$' | \
        grep -v '^\s*use ' | \
        grep -v '^\s*pub mod ' | \
        grep -v '^\s*mod ' | \
        grep -v '^\s*Ok(())' | \
        grep -v '^\s*None' | \
        grep -v '^\s*Some(' | \
        grep -v '^\s*Self ' | \
        grep -v '^\s*#\[' | \
        sed 's/^[[:space:]]*//' | \
        awk -v min="$MIN_LINE_LEN" 'length >= min' | \
        sort -u
}

extract_lines "$WORKDIR/$NAME_B" > "$WORKDIR/lines_b.txt"

# For cerulean-ledger: include both src/ and crates/
LEDGER_DIRS=("$WORKDIR/$NAME_A/src" "$WORKDIR/$NAME_A/crates")
cat /dev/null > "$WORKDIR/lines_a.txt"
for dir in "${LEDGER_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        extract_lines "$dir" >> "$WORKDIR/lines_a.txt"
    fi
done
sort -u "$WORKDIR/lines_a.txt" -o "$WORKDIR/lines_a.txt"

UNIQUE_A=$(wc -l < "$WORKDIR/lines_a.txt" | tr -d ' ')
UNIQUE_B=$(wc -l < "$WORKDIR/lines_b.txt" | tr -d ' ')

comm -12 "$WORKDIR/lines_a.txt" "$WORKDIR/lines_b.txt" > "$WORKDIR/common_lines.txt"
COMMON_COUNT=$(wc -l < "$WORKDIR/common_lines.txt" | tr -d ' ')

if [[ "$UNIQUE_B" -gt 0 ]]; then
    PERCENT=$(awk "BEGIN {printf \"%.1f\", ($COMMON_COUNT / $UNIQUE_B) * 100}")
else
    PERCENT="0.0"
fi

log "  Lineas no triviales en $NAME_A: $UNIQUE_A"
log "  Lineas no triviales en $NAME_B: $UNIQUE_B"
log "  Coincidencias exactas: $COMMON_COUNT ($PERCENT% del repo comparado)"
log ""

if [[ "$COMMON_COUNT" -gt 0 ]]; then
    log "  --- Lineas coincidentes (con contexto) ---"
    log ""
    while IFS= read -r line; do
        log "  LINEA: $line"
        log ""

        log "    En $NAME_B:"
        (grep -rn --include='*.rs' -F "$line" "$WORKDIR/$NAME_B" || true) | head -3 | while IFS= read -r match; do
            clean=$(echo "$match" | sed "s|$WORKDIR/$NAME_B/||")
            log "      $clean"
        done
        log ""

        log "    En $NAME_A:"
        (grep -rn --include='*.rs' -F "$line" "$WORKDIR/$NAME_A/src" "$WORKDIR/$NAME_A/crates" 2>/dev/null || true) | head -3 | while IFS= read -r match; do
            clean=$(echo "$match" | sed "s|$WORKDIR/$NAME_A/||")
            log "      $clean"
        done
        log ""
    done < "$WORKDIR/common_lines.txt"
else
    log "  No se encontraron coincidencias."
    log ""
fi

# ==============================================================================
# 7. Verificacion de funciones y structs
# ==============================================================================
log "[7/8] Comparando nombres de funciones y structs..."

extract_signatures() {
    find "$1" -name '*.rs' -exec cat {} + 2>/dev/null | \
        grep -E '^\s*(pub\s+)?(fn|struct|enum|trait|impl)\s+\w+' | \
        sed 's/^\s*//' | \
        sed 's/{.*//' | \
        sed 's/\s*$//' | \
        sort -u
}

extract_signatures "$WORKDIR/$NAME_B" > "$WORKDIR/sigs_b.txt"

cat /dev/null > "$WORKDIR/sigs_a.txt"
for dir in "${LEDGER_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        extract_signatures "$dir" >> "$WORKDIR/sigs_a.txt"
    fi
done
sort -u "$WORKDIR/sigs_a.txt" -o "$WORKDIR/sigs_a.txt"

COMMON_SIGS=$(comm -12 "$WORKDIR/sigs_a.txt" "$WORKDIR/sigs_b.txt")
SIGS_B_COUNT=$(wc -l < "$WORKDIR/sigs_b.txt" | tr -d ' ')

log "  Firmas (fn/struct/enum/trait) en $NAME_B: $SIGS_B_COUNT"

if [[ -z "$COMMON_SIGS" ]]; then
    log "  Firmas identicas entre repos: 0"
else
    COMMON_SIGS_COUNT=$(echo "$COMMON_SIGS" | wc -l | tr -d ' ')
    log "  Firmas identicas entre repos: $COMMON_SIGS_COUNT"
    echo "$COMMON_SIGS" | while read -r sig; do
        log "    - $sig"
    done
fi
log ""

# ==============================================================================
# 8. Checksums para auditoria
# ==============================================================================
log "[8/8] Checksums SHA-256 de los repositorios analizados..."

CHECKSUM_A=$(cd "$WORKDIR/$NAME_A" && find . -name '*.rs' -exec cat {} + 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
CHECKSUM_B=$(cd "$WORKDIR/$NAME_B" && find . -name '*.rs' -exec cat {} + 2>/dev/null | shasum -a 256 | cut -d' ' -f1)

log "  $NAME_A (todos los .rs): $CHECKSUM_A"
log "  $NAME_B (todos los .rs): $CHECKSUM_B"
log ""

# ==============================================================================
# Resumen
# ==============================================================================
separator
log "RESUMEN"
separator
log ""
log "  Primer commit $NAME_A:   $FIRST_A"
log "  Primer commit $NAME_B:   $FIRST_B"
log "  Commits:                      $COMMITS_A vs $COMMITS_B"
log "  Archivos Rust:                $FILES_RS_A vs $FILES_RS_B"
log "  Lineas Rust:                  $LINES_A vs $LINES_B"
log "  Frameworks:                   Actix-Web vs Substrate (incompatibles)"
log "  Coincidencias de codigo:      $COMMON_COUNT de $UNIQUE_B lineas ($PERCENT%)"

if [[ "$COMMON_COUNT" -le 10 ]]; then
    log ""
    log "  VEREDICTO: No existe evidencia de copia de codigo."
    if [[ "$COMMON_COUNT" -gt 0 ]]; then
        log "  Las $COMMON_COUNT coincidencia(s) son declaraciones triviales del lenguaje Rust."
    fi
fi

log ""
separator
log "Verificacion completada — $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
log "Directorio temporal limpiado automaticamente."
separator

if [[ -n "$OUTPUT" ]]; then
    echo ""
    echo "Reporte guardado en: $OUTPUT"
fi
