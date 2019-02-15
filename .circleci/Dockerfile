FROM circleci/rust:1.32
COPY jormungandr node_config.yml genesis.json key.xprv ./
CMD sudo chmod 777 /data && ./jormungandr --config node_config.yml --genesis-config genesis.json --secret key.xprv --storage /data -vvv
