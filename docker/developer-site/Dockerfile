FROM node:14-buster-slim AS toolchain

# To use http/https proxy while building, use:
# docker build --build-arg https_proxy=http://fwdproxy:8080 --build-arg http_proxy=http://fwdproxy:8080

# to be run from within developers.diem.com dir
COPY . /diem

WORKDIR /diem

RUN    yarn \
    && apt-get update \
    && apt-get install python3.7 python3.7-distutils curl git --yes \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py \
    && python3.7 get-pip.py \
    && rm -rf get-pip.py \
    && pip3 install git+https://github.com/linkchecker/linkchecker.git

CMD ["--url", "https://testnet.diem.com/v1", "--waypoint_url",  "https://testnet.diem.com/waypoint.txt", "--chain-id", "TESTNET"]
