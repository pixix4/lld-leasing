#!/bin/bash

set -x -a -e
trap "echo Unexpected error! See log above; exit 1" ERR

# CONFIG Parameters (might change)

export IMAGE=${IMAGE:-lld_server_image}
export SCONE_CAS_ADDR="4-2-1.scone-cas.cf"
export DEVICE="/dev/sgx"

export CAS_MRENCLAVE="4cd0fe54d3d8d787553b7dac7347012682c402220acd062e4d0da3bbe10a1c2c"

export CLI_IMAGE="registry.scontain.com:5050/sconecuratedimages/kubernetes:hello-k8s-scone0.1"
export LLD_IMAGE="pixix4/lld-scone-sqlite"
export LLD_MRENCLAVE="67b8017f7083435cb614b87c8daa14303f741a10a2a0bbf5dfabec777cf629b9"

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


# attest cas before uploading the session file, accept CAS running in debug
# mode (-d) and outdated TCB (-G)
docker run --device=$DEVICE -it $CLI_IMAGE sh -c "
scone cas attest -G --only_for_testing-debug  $SCONE_CAS_ADDR $CAS_MRENCLAVE >/dev/null \
&&  scone cas show-certificate" > cas-ca.pem

# ensure that we have self-signed client certificate

if [[ ! -f client.pem || ! -f client-key.pem  ]] ; then
    openssl req -newkey rsa:4096 -days 365 -nodes -x509 -out client.pem -keyout client-key.pem -config clientcertreq.conf
fi

# create session file

MRENCLAVE=$LLD_MRENCLAVE envsubst '$MRENCLAVE $LLD_SESSION' < lld-template.yml > lld_session.yml
# note: this is insecure - use scone session create instead
curl -v -k -s --cert client.pem  --key client-key.pem  --data-binary @redis_session.yml -X POST https://$SCONE_CAS_ADDR:8081/session

# create file with environment variables

cat > myenv << EOF
export LLD_SESSION="$LLD_SESSION"
export SCONE_CAS_ADDR="$SCONE_CAS_ADDR"
export IMAGE="$IMAGE"
export DEVICE="$DEVICE"

EOF

echo "OK"
