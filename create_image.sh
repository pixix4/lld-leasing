#!/bin/bash

set -x -a -e
trap "echo Unexpected error! See log above; exit 1" ERR

shopt -s expand_aliases
export ALIAS="$HOME/.bash_aliases"
source "$ALIAS"
type -a scone || error "alias 'scone' undefined. Please add this to your .bashrc first."


# CONFIG Parameters (might change)

export SCONE_CAS_ADDR="5-7-0.scone-cas.cf"
export DEVICE="/dev/sgx"

export CAS_MRENCLAVE="3061b9feb7fa67f3815336a085f629a13f04b0a1667c93b14ff35581dc8271e4"

export CLI_IMAGE="registry.scontain.com:5050/community/cli"
export SERVER_SQLITE_MRENCLAVE="1c8c1c975236e6c6d735046e36f124c62f22ade890dfb77a3185f93954c3448a"
export SERVER_DQLITE_MRENCLAVE="ca54ff2b96644b051f92bbe71c47e24c3aefa044367b5e319f6f0e56ed12fe22"
export DQLITE_MRENCLAVE="c93961b7cbe32f43a1fa2cfe52f9d58eaaa098018c6592141f0a29e6450fc512"

# create random and hence, uniquee session number
LLD_SESSION="LldSession-$RANDOM-$RANDOM-$RANDOM"

# ensure that we have an up-to-date image
docker pull $CLI_IMAGE

# check if SGX device exists

if [[ ! -c "$DEVICE" ]] ; then
    export DEVICE_O="DEVICE"
    export DEVICE="/dev/isgx"
    if [[ ! -c "$DEVICE" ]] ; then
        echo "Neither $DEVICE_O nor $DEVICE exist"
        exit 1
    fi
fi

# curl -k -X POST -H "Content-Type: application/json" -d '{"application_id": "1", "instance_id": "1", "duration": 5000}' https://localhost:3030/request

# attest cas before uploading the session file, accept CAS running in debug
# mode (-d) and outdated TCB (-G)
scone cas attest -G --only_for_testing-debug --only_for_testing-ignore-signer $SCONE_CAS_ADDR $CAS_MRENCLAVE
scone cas show-certificate > cas-ca.pem

# ensure that we have self-signed client certificate

if [[ ! -f client.pem || ! -f client-key.pem  ]] ; then
    openssl req -newkey rsa:4096 -days 365 -nodes -x509 -out client.pem -keyout client-key.pem -config clientcertreq.conf
fi

# create session file

envsubst '$SERVER_SQLITE_MRENCLAVE $SERVER_DQLITE_MRENCLAVE $DQLITE_MRENCLAVE $LLD_SESSION' < lld-template.yml > lld_session.yml
# note: this is insecure - use scone session create instead
curl -v -k -s --cert client.pem  --key client-key.pem  --data-binary @lld_session.yml -X POST https://$SCONE_CAS_ADDR:8081/session

# create file with environment variables

cat > myenv << EOF
export LLD_SESSION="$LLD_SESSION"
export SCONE_CAS_ADDR="$SCONE_CAS_ADDR"
export DEVICE="$DEVICE"
EOF

# curl -k -X GET "https://${SCONE_CAS_ADDR}:8081/v1/values/session=$LLD_SESSION" | jq -r .values.api_ca_cert.value > cacert.pem

echo "OK"
