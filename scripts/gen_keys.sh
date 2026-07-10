#!/usr/bin/env bash
# Генерация пары ключей Ed25519 для подписи JWT (EdDSA).
# Приватный ключ (PKCS#8) остаётся только в auth-сервисе; публичный (SPKI)
# раздаётся сервисам-клиентам для оффлайн-валидации токенов.
#
# Использование:  ./scripts/gen_keys.sh [OUT_DIR]
set -euo pipefail

OUT_DIR="${1:-keys}"
mkdir -p "$OUT_DIR"

openssl genpkey -algorithm ed25519 -out "$OUT_DIR/ed25519_private.pem"
openssl pkey -in "$OUT_DIR/ed25519_private.pem" -pubout -out "$OUT_DIR/ed25519_public.pem"

echo "Ed25519 keypair written to:"
echo "  $OUT_DIR/ed25519_private.pem  (secret — auth-service only)"
echo "  $OUT_DIR/ed25519_public.pem   (share with client services)"
