#!/bin/bash
set -e

# Configuration
CERT_DIR="tests/resources/certs"
mkdir -p "$CERT_DIR"

echo "Generating mTLS test certificates in $CERT_DIR..."

# 1. Generate Root CA
openssl genpkey -algorithm ed25519 -out "$CERT_DIR/ca.key"
chmod 600 "$CERT_DIR/ca.key"
openssl req -new -x509 -key "$CERT_DIR/ca.key" -out "$CERT_DIR/ca.pem" -days 3650 -subj "/CN=Wazuh Test Root CA" -addext "basicConstraints=critical,CA:TRUE"

# 2. Generate Server Key and CSR
openssl genpkey -algorithm ed25519 -out "$CERT_DIR/server.key"
chmod 600 "$CERT_DIR/server.key"
openssl req -new -key "$CERT_DIR/server.key" -out "$CERT_DIR/server.csr" -subj "/CN=localhost"

# 3. Sign Server Certificate
cat > "$CERT_DIR/server.ext" <<EOF
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = DNS:localhost, IP:127.0.0.1
EOF
openssl x509 -req -in "$CERT_DIR/server.csr" -CA "$CERT_DIR/ca.pem" -CAkey "$CERT_DIR/ca.key" -CAcreateserial -out "$CERT_DIR/server.pem" -days 365 -extfile "$CERT_DIR/server.ext"

# 4. Generate Client Key and CSR
openssl genpkey -algorithm ed25519 -out "$CERT_DIR/client.key"
chmod 600 "$CERT_DIR/client.key"
openssl req -new -key "$CERT_DIR/client.key" -out "$CERT_DIR/client.csr" -subj "/CN=WazuhTrayClient"

# 5. Sign Client Certificate
cat > "$CERT_DIR/client.ext" <<EOF
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature
extendedKeyUsage = clientAuth
EOF
openssl x509 -req -in "$CERT_DIR/client.csr" -CA "$CERT_DIR/ca.pem" -CAkey "$CERT_DIR/ca.key" -CAcreateserial -out "$CERT_DIR/client.pem" -days 365 -extfile "$CERT_DIR/client.ext"

# 6. Generate an Untrusted CA and Client
mkdir -p "$CERT_DIR/untrusted"
openssl genpkey -algorithm ed25519 -out "$CERT_DIR/untrusted/ca.key"
chmod 600 "$CERT_DIR/untrusted/ca.key"
openssl req -new -x509 -key "$CERT_DIR/untrusted/ca.key" -out "$CERT_DIR/untrusted/ca.pem" -days 3650 -subj "/CN=Untrusted CA" -addext "basicConstraints=critical,CA:TRUE"

# Untrusted Client
openssl genpkey -algorithm ed25519 -out "$CERT_DIR/untrusted/client.key"
chmod 600 "$CERT_DIR/untrusted/client.key"
openssl req -new -key "$CERT_DIR/untrusted/client.key" -out "$CERT_DIR/untrusted/client.csr" -subj "/CN=UntrustedClient"
openssl x509 -req -in "$CERT_DIR/untrusted/client.csr" -CA "$CERT_DIR/untrusted/ca.pem" -CAkey "$CERT_DIR/untrusted/ca.key" -CAcreateserial -out "$CERT_DIR/untrusted/client.pem" -days 365 -extfile "$CERT_DIR/client.ext"

# Clean up
rm "$CERT_DIR"/*.csr "$CERT_DIR"/*.ext "$CERT_DIR/untrusted"/*.csr "$CERT_DIR/untrusted"/*.ext || true
rm "$CERT_DIR"/*.srl "$CERT_DIR/untrusted"/*.srl || true

echo "Done. Certificates generated successfully with secure permissions."
