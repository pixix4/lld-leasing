#!/bin/bash

rm -rf certificates/
mkdir certificates/
cd certificates/

#openssl ecparam -out root.key -name prime256v1 -genkey
openssl genrsa -out root.key 3072
openssl req -new -sha256 -key root.key -out root.csr -subj "/C=de/CN=mac.local"
openssl x509 -req -sha256 -days 365 -in root.csr -signkey root.key -out root.crt

#openssl ecparam -out lld-server.key -name prime256v1 -genkey
openssl genrsa -out lld-server.key 3072
openssl req -new -sha256 -key lld-server.key -out lld-server.csr -subj "/C=de/CN=mac.local"
openssl x509 -req -in lld-server.csr -CA  root.crt -CAkey root.key -CAcreateserial -out lld-server.crt -days 365 -sha256
openssl x509 -in lld-server.crt -text -noout
